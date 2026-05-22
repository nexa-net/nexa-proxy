use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub http_listen: String,
    pub https_listen: Option<String>,
    #[serde(default)]
    pub routes: HashMap<String, ProxyRouteConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyRouteConfig {
    pub upstreams: Vec<UpstreamEntry>,
    #[serde(default)]
    pub tls: Option<TlsEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpstreamEntry {
    pub address: String,
    pub weight: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsEntry {
    pub cert_path: Option<PathBuf>,
    pub key_path: Option<PathBuf>,
    pub acme_email: Option<String>,
}

impl ProxyConfig {
    pub fn load(path: &std::path::Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = serde_json::from_str(&content)?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_config() {
        let json = r#"{
            "http_listen": "0.0.0.0:80",
            "routes": {}
        }"#;
        let config: ProxyConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.http_listen, "0.0.0.0:80");
        assert!(config.routes.is_empty());
    }

    #[test]
    fn parse_full_config() {
        let json = r#"{
            "http_listen": "0.0.0.0:80",
            "https_listen": "0.0.0.0:443",
            "routes": {
                "api.example.com": {
                    "upstreams": [
                        {"address": "10.0.0.1:3000", "weight": 1},
                        {"address": "10.0.0.2:3000", "weight": 2}
                    ],
                    "tls": {
                        "acme_email": "admin@example.com"
                    }
                }
            }
        }"#;
        let config: ProxyConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.routes.len(), 1);
        let route = &config.routes["api.example.com"];
        assert_eq!(route.upstreams.len(), 2);
        assert_eq!(route.upstreams[1].weight, 2);
        assert_eq!(
            route.tls.as_ref().unwrap().acme_email.as_deref(),
            Some("admin@example.com")
        );
    }

    #[test]
    fn config_serializes_roundtrip() {
        let config = ProxyConfig {
            http_listen: "0.0.0.0:80".into(),
            https_listen: None,
            routes: HashMap::new(),
        };
        let json = serde_json::to_string(&config).unwrap();
        let deser: ProxyConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.http_listen, "0.0.0.0:80");
    }
}
