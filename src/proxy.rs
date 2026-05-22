use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use tokio::net::TcpListener;
use tracing::{error, info};

use crate::config::ProxyConfig;

pub struct ProxyState {
    pub routes: HashMap<String, RouteState>,
}

pub struct RouteState {
    pub upstreams: Vec<WeightedUpstream>,
    pub counter: AtomicUsize,
}

pub struct WeightedUpstream {
    pub address: String,
    pub weight: u32,
}

impl ProxyState {
    pub fn from_config(config: &ProxyConfig) -> Self {
        let mut routes = HashMap::new();
        for (domain, route_config) in &config.routes {
            let upstreams: Vec<WeightedUpstream> = route_config
                .upstreams
                .iter()
                .map(|u| WeightedUpstream {
                    address: u.address.clone(),
                    weight: u.weight,
                })
                .collect();
            routes.insert(
                domain.clone(),
                RouteState {
                    upstreams,
                    counter: AtomicUsize::new(0),
                },
            );
        }
        Self { routes }
    }

    pub fn select_upstream(&self, domain: &str) -> Option<&str> {
        let route = self.routes.get(domain)?;
        if route.upstreams.is_empty() {
            return None;
        }

        let total_weight: u32 = route.upstreams.iter().map(|u| u.weight).sum();
        if total_weight == 0 {
            return None;
        }

        let idx = route.counter.fetch_add(1, Ordering::Relaxed);
        let mut target = (idx as u32) % total_weight;

        for upstream in &route.upstreams {
            if target < upstream.weight {
                return Some(&upstream.address);
            }
            target -= upstream.weight;
        }

        Some(&route.upstreams[0].address)
    }
}

pub async fn run_http(listen_addr: &str, state: Arc<ProxyState>) -> anyhow::Result<()> {
    let listener = TcpListener::bind(listen_addr).await?;
    info!(%listen_addr, "nexa-proxy HTTP listening");

    loop {
        let (stream, peer_addr) = listener.accept().await?;
        let state = state.clone();

        tokio::spawn(async move {
            let service = service_fn(move |req: Request<Incoming>| {
                let state = state.clone();
                async move { handle_request(req, &state).await }
            });

            if let Err(e) = http1::Builder::new()
                .serve_connection(hyper_util::rt::TokioIo::new(stream), service)
                .await
            {
                error!(%peer_addr, %e, "connection error");
            }
        });
    }
}

async fn handle_request(
    req: Request<Incoming>,
    state: &ProxyState,
) -> std::result::Result<Response<Full<Bytes>>, hyper::Error> {
    let host = req
        .headers()
        .get("host")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .split(':')
        .next()
        .unwrap_or("");

    let upstream = match state.select_upstream(host) {
        Some(addr) => addr.to_string(),
        None => {
            return Ok(Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .body(Full::new(Bytes::from(
                    "no upstream configured for this domain",
                )))
                .unwrap());
        }
    };

    let uri = format!(
        "http://{}{}",
        upstream,
        req.uri()
            .path_and_query()
            .map(|pq| pq.as_str())
            .unwrap_or("/")
    );

    let parts = req.into_parts().0;
    let method = match parts.method.as_str() {
        "GET" => reqwest::Method::GET,
        "POST" => reqwest::Method::POST,
        "PUT" => reqwest::Method::PUT,
        "DELETE" => reqwest::Method::DELETE,
        "PATCH" => reqwest::Method::PATCH,
        "HEAD" => reqwest::Method::HEAD,
        "OPTIONS" => reqwest::Method::OPTIONS,
        _ => reqwest::Method::GET,
    };

    let client = reqwest::Client::new();
    let mut builder = client.request(method, &uri);

    for (name, value) in &parts.headers {
        if name != "host" && name != "connection" {
            if let Ok(v) = value.to_str() {
                builder = builder.header(name.as_str(), v);
            }
        }
    }

    match builder.send().await {
        Ok(upstream_resp) => {
            let status = StatusCode::from_u16(upstream_resp.status().as_u16())
                .unwrap_or(StatusCode::BAD_GATEWAY);
            let body_bytes = upstream_resp.bytes().await.unwrap_or_default();

            Ok(Response::builder()
                .status(status)
                .body(Full::new(body_bytes))
                .unwrap())
        }
        Err(e) => {
            error!(%upstream, %e, "upstream request failed");
            Ok(Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .body(Full::new(Bytes::from(format!("upstream error: {e}"))))
                .unwrap())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ProxyConfig, ProxyRouteConfig, UpstreamEntry};

    fn make_state() -> ProxyState {
        let config = ProxyConfig {
            http_listen: "0.0.0.0:80".into(),
            https_listen: None,
            routes: HashMap::from([
                (
                    "api.example.com".into(),
                    ProxyRouteConfig {
                        upstreams: vec![
                            UpstreamEntry {
                                address: "10.0.0.1:3000".into(),
                                weight: 1,
                            },
                            UpstreamEntry {
                                address: "10.0.0.2:3000".into(),
                                weight: 2,
                            },
                        ],
                        tls: None,
                    },
                ),
                (
                    "web.example.com".into(),
                    ProxyRouteConfig {
                        upstreams: vec![UpstreamEntry {
                            address: "10.0.0.5:80".into(),
                            weight: 1,
                        }],
                        tls: None,
                    },
                ),
            ]),
        };
        ProxyState::from_config(&config)
    }

    #[test]
    fn select_upstream_known_domain() {
        let state = make_state();
        let upstream = state.select_upstream("web.example.com");
        assert_eq!(upstream, Some("10.0.0.5:80"));
    }

    #[test]
    fn select_upstream_unknown_domain() {
        let state = make_state();
        assert!(state.select_upstream("unknown.example.com").is_none());
    }

    #[test]
    fn weighted_round_robin() {
        let state = make_state();
        let first = state.select_upstream("api.example.com").unwrap();
        assert_eq!(first, "10.0.0.1:3000");

        let second = state.select_upstream("api.example.com").unwrap();
        assert_eq!(second, "10.0.0.2:3000");

        let third = state.select_upstream("api.example.com").unwrap();
        assert_eq!(third, "10.0.0.2:3000");

        let fourth = state.select_upstream("api.example.com").unwrap();
        assert_eq!(fourth, "10.0.0.1:3000");
    }

    #[test]
    fn from_config_empty_routes() {
        let config = ProxyConfig {
            http_listen: "0.0.0.0:80".into(),
            https_listen: None,
            routes: HashMap::new(),
        };
        let state = ProxyState::from_config(&config);
        assert!(state.routes.is_empty());
    }
}
