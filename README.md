<div align="center">

# nexa-proxy

**NexaNet built-in reverse proxy -- minimal HTTP load balancer with weighted routing**

[![CI](https://github.com/nexa-net/nexa-proxy/actions/workflows/ci.yml/badge.svg)](https://github.com/nexa-net/nexa-proxy/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org)

nexa-proxy is a lightweight reverse proxy built on hyper 1.x. It routes incoming
HTTP requests to backend containers based on the `Host` header, using weighted
round-robin load balancing. nexad manages its config file automatically, but it
can also run standalone.

</div>

---

## Features

- **HTTP/1.1 and HTTP/2** reverse proxying via hyper 1.x
- **Weighted round-robin** load balancing across multiple upstreams
- **Host-based routing** -- routes requests by domain name
- **JSON config file** -- simple, human-readable route definitions
- **HTTPS support** -- optional TLS termination with ACME or manual certificates
- **Minimal footprint** -- single binary, no external dependencies at runtime
- **7 unit tests**

## Quick Start

### Build

```bash
cargo build --release
# Binary is at ./target/release/nexa-proxy
```

### Run

```bash
# Start with a config file
nexa-proxy --config /path/to/proxy.json
```

## Configuration

nexa-proxy reads a JSON config file that defines listen addresses and route mappings.

### Minimal config

```json
{
    "http_listen": "0.0.0.0:80",
    "routes": {}
}
```

### Full config with weighted upstreams and TLS

```json
{
    "http_listen": "0.0.0.0:80",
    "https_listen": "0.0.0.0:443",
    "routes": {
        "api.example.com": {
            "upstreams": [
                { "address": "10.0.0.1:3000", "weight": 2 },
                { "address": "10.0.0.2:3000", "weight": 1 }
            ],
            "tls": {
                "acme_email": "admin@example.com"
            }
        },
        "web.example.com": {
            "upstreams": [
                { "address": "10.0.0.5:8080", "weight": 1 }
            ]
        }
    }
}
```

### Config reference

| Field | Type | Required | Description |
|---|---|---|---|
| `http_listen` | string | yes | HTTP listen address (e.g. `0.0.0.0:80`) |
| `https_listen` | string | no | HTTPS listen address (e.g. `0.0.0.0:443`) |
| `routes` | object | yes | Map of domain to route config |
| `routes.<domain>.upstreams` | array | yes | Backend servers |
| `routes.<domain>.upstreams[].address` | string | yes | Backend address (`host:port`) |
| `routes.<domain>.upstreams[].weight` | integer | yes | Relative weight for load balancing |
| `routes.<domain>.tls` | object | no | TLS configuration |
| `routes.<domain>.tls.cert_path` | string | no | Path to certificate PEM file |
| `routes.<domain>.tls.key_path` | string | no | Path to private key PEM file |
| `routes.<domain>.tls.acme_email` | string | no | Email for ACME auto-provisioning |

### Load balancing

Requests are distributed across upstreams using weighted round-robin. A backend with
`weight: 2` receives twice as many requests as one with `weight: 1`.

```
                          weight: 2
  Client ──> nexa-proxy ──────────────> 10.0.0.1:3000
         (Host: api.example.com)
                          weight: 1
                       ──────────────> 10.0.0.2:3000
```

## Architecture

```
nexa-proxy/
  src/
    main.rs      -- CLI entry point, config loading, server startup
    config.rs    -- ProxyConfig, ProxyRouteConfig, UpstreamEntry, TlsEntry
    proxy.rs     -- ProxyState, weighted round-robin selection, HTTP handler
```

The proxy is intentionally minimal:

1. Bind to the configured listen address
2. For each incoming request, extract the `Host` header
3. Look up the matching route
4. Select an upstream via weighted round-robin
5. Forward the request and stream the response back

## Integration with nexad

When nexad is configured with `--proxy-backend nexa-proxy` (the default), it
automatically generates and updates the `proxy.json` config file as routes are
added or removed. nexa-proxy picks up changes on the next request.

```bash
# nexad manages the config
nexad --proxy-backend nexa-proxy --proxy-config-dir /var/lib/nexa/proxy

# nexa-proxy reads it
nexa-proxy --config /var/lib/nexa/proxy/proxy.json
```

## CLI Flags

```
nexa-proxy [OPTIONS]

Options:
    --config <PATH>    Path to proxy config file [default: /var/lib/nexa/proxy.json]
    --help             Show help
    --version          Show version
```

## Development

```bash
# Build
cargo build

# Run tests
cargo test

# Build release binary
cargo build --release
```

## Related Repositories

| Repository | Description |
|---|---|
| [nexa-core](https://github.com/nexa-net/nexa-core) | Core domain types, traits, and orchestrator |
| [nexad](https://github.com/nexa-net/nexad) | Daemon -- container runtime, state, API, clustering |
| [nexa-cli](https://github.com/nexa-net/nexa-cli) | CLI tool for deploying and managing containers |

## License

Apache-2.0 -- see [LICENSE](LICENSE) for details.
