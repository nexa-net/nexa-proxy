use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;

use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use nexa_proxy::config::{ProxyConfig, ProxyRouteConfig, UpstreamEntry};
use nexa_proxy::proxy::ProxyState;
use tokio::net::TcpListener;

/// Spawn a minimal HTTP backend that always returns `body_id` as the response body.
/// Returns the bound SocketAddr.
async fn spawn_backend(body_id: &'static str) -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        loop {
            let (stream, _) = listener.accept().await.unwrap();
            tokio::spawn(async move {
                let svc = service_fn(move |_req: Request<Incoming>| async move {
                    Ok::<Response<Full<Bytes>>, Infallible>(
                        Response::new(Full::new(Bytes::from(body_id))),
                    )
                });
                let _ = http1::Builder::new()
                    .serve_connection(TokioIo::new(stream), svc)
                    .await;
            });
        }
    });

    addr
}

/// Build a ProxyState from config and start the proxy on a random port.
/// Returns the bound SocketAddr.
async fn spawn_proxy(config: ProxyConfig) -> SocketAddr {
    let listen = "127.0.0.1:0";
    // Bind first so we can get the address before handing off to run_http.
    let listener = TcpListener::bind(listen).await.unwrap();
    let addr = listener.local_addr().unwrap();

    let state = Arc::new(ProxyState::from_config(&config));

    tokio::spawn(async move {
        // run_http binds its own listener, so we replicate the serve loop here
        // using the pre-bound listener.
        loop {
            let (stream, peer_addr) = listener.accept().await.unwrap();
            let state = state.clone();
            tokio::spawn(async move {
                let svc = service_fn(move |req: Request<Incoming>| {
                    let state = state.clone();
                    async move { handle_req_via_state(req, &state).await }
                });
                if let Err(e) = http1::Builder::new()
                    .serve_connection(TokioIo::new(stream), svc)
                    .await
                {
                    eprintln!("proxy conn error from {peer_addr}: {e}");
                }
            });
        }
    });

    addr
}

/// Mirrors the handle_request logic from proxy.rs so the integration tests
/// drive the actual ProxyState routing without re-importing the private fn.
async fn handle_req_via_state(
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
                .status(hyper::StatusCode::BAD_GATEWAY)
                .body(Full::new(Bytes::from("no upstream")))
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

    let client = reqwest::Client::new();
    match client.get(&uri).send().await {
        Ok(resp) => {
            let status =
                hyper::StatusCode::from_u16(resp.status().as_u16()).unwrap_or(hyper::StatusCode::BAD_GATEWAY);
            let body = resp.bytes().await.unwrap_or_default();
            Ok(Response::builder()
                .status(status)
                .body(Full::new(body))
                .unwrap())
        }
        Err(e) => Ok(Response::builder()
            .status(hyper::StatusCode::BAD_GATEWAY)
            .body(Full::new(Bytes::from(format!("upstream error: {e}"))))
            .unwrap()),
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[tokio::test]
async fn route_to_single_upstream() {
    let backend = spawn_backend("hello-backend").await;

    let mut routes = HashMap::new();
    routes.insert(
        "app.test".into(),
        ProxyRouteConfig {
            upstreams: vec![UpstreamEntry {
                address: backend.to_string(),
                weight: 1,
            }],
            tls: None,
        },
    );

    let config = ProxyConfig {
        http_listen: "127.0.0.1:0".into(),
        https_listen: None,
        routes,
    };

    let proxy_addr = spawn_proxy(config).await;

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://{proxy_addr}/"))
        .header("host", "app.test")
        .send()
        .await
        .expect("request failed");

    assert_eq!(resp.status().as_u16(), 200);
    let body = resp.text().await.unwrap();
    assert_eq!(body, "hello-backend");
}

#[tokio::test]
async fn unknown_host_returns_502() {
    let config = ProxyConfig {
        http_listen: "127.0.0.1:0".into(),
        https_listen: None,
        routes: HashMap::new(),
    };

    let proxy_addr = spawn_proxy(config).await;

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://{proxy_addr}/"))
        .header("host", "unknown.test")
        .send()
        .await
        .expect("request failed");

    assert_eq!(resp.status().as_u16(), 502);
}

#[tokio::test]
async fn weighted_round_robin_distribution() {
    let backend_a = spawn_backend("A").await;
    let backend_b = spawn_backend("B").await;

    let mut routes = HashMap::new();
    routes.insert(
        "rr.test".into(),
        ProxyRouteConfig {
            upstreams: vec![
                UpstreamEntry {
                    address: backend_a.to_string(),
                    weight: 1,
                },
                UpstreamEntry {
                    address: backend_b.to_string(),
                    weight: 2,
                },
            ],
            tls: None,
        },
    );

    let config = ProxyConfig {
        http_listen: "127.0.0.1:0".into(),
        https_listen: None,
        routes,
    };

    let proxy_addr = spawn_proxy(config).await;

    let client = reqwest::Client::new();
    let total = 90usize;
    let mut count_a = 0usize;
    let mut count_b = 0usize;

    for _ in 0..total {
        let body = client
            .get(format!("http://{proxy_addr}/"))
            .header("host", "rr.test")
            .send()
            .await
            .expect("request failed")
            .text()
            .await
            .unwrap();

        match body.as_str() {
            "A" => count_a += 1,
            "B" => count_b += 1,
            other => panic!("unexpected body: {other}"),
        }
    }

    // Weight 1:2 → A should get ~30, B ~60. Allow ±10 tolerance.
    assert!(
        (25..=35).contains(&count_a),
        "backend A got {count_a} hits (expected ~30)"
    );
    assert!(
        (55..=65).contains(&count_b),
        "backend B got {count_b} hits (expected ~60)"
    );
}

#[tokio::test]
async fn multiple_domains_route_independently() {
    let backend_x = spawn_backend("domain-x").await;
    let backend_y = spawn_backend("domain-y").await;

    let mut routes = HashMap::new();
    routes.insert(
        "x.test".into(),
        ProxyRouteConfig {
            upstreams: vec![UpstreamEntry {
                address: backend_x.to_string(),
                weight: 1,
            }],
            tls: None,
        },
    );
    routes.insert(
        "y.test".into(),
        ProxyRouteConfig {
            upstreams: vec![UpstreamEntry {
                address: backend_y.to_string(),
                weight: 1,
            }],
            tls: None,
        },
    );

    let config = ProxyConfig {
        http_listen: "127.0.0.1:0".into(),
        https_listen: None,
        routes,
    };

    let proxy_addr = spawn_proxy(config).await;
    let client = reqwest::Client::new();

    let body_x = client
        .get(format!("http://{proxy_addr}/"))
        .header("host", "x.test")
        .send()
        .await
        .expect("request for x.test failed")
        .text()
        .await
        .unwrap();

    let body_y = client
        .get(format!("http://{proxy_addr}/"))
        .header("host", "y.test")
        .send()
        .await
        .expect("request for y.test failed")
        .text()
        .await
        .unwrap();

    assert_eq!(body_x, "domain-x", "x.test should route to backend_x");
    assert_eq!(body_y, "domain-y", "y.test should route to backend_y");
}
