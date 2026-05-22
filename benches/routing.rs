use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use nexa_proxy::config::{ProxyConfig, ProxyRouteConfig, UpstreamEntry};
use nexa_proxy::proxy::ProxyState;
use std::collections::HashMap;

fn make_config(num_routes: usize, upstreams_per_route: usize) -> ProxyConfig {
    let mut routes = HashMap::new();
    for i in 0..num_routes {
        let domain = format!("domain-{i}.example.com");
        let upstreams = (0..upstreams_per_route)
            .map(|j| UpstreamEntry {
                address: format!("10.0.{}.{}:8080", i % 256, j % 256),
                weight: ((j % 3) + 1) as u32,
            })
            .collect();
        routes.insert(
            domain,
            ProxyRouteConfig {
                upstreams,
                tls: None,
            },
        );
    }
    ProxyConfig {
        http_listen: "0.0.0.0:80".into(),
        https_listen: None,
        routes,
    }
}

fn bench_select_upstream_n_routes(c: &mut Criterion) {
    let mut group = c.benchmark_group("select_upstream");
    for &num_routes in &[10usize, 100, 1000] {
        let config = make_config(num_routes, 3);
        let state = ProxyState::from_config(&config);
        group.bench_with_input(
            BenchmarkId::new("routes", num_routes),
            &num_routes,
            |b, _| {
                b.iter(|| state.select_upstream(black_box("domain-0.example.com")));
            },
        );
    }
    group.finish();
}

fn bench_weighted_round_robin(c: &mut Criterion) {
    let mut group = c.benchmark_group("weighted_round_robin");
    for &num_upstreams in &[3usize, 10] {
        let config = make_config(1, num_upstreams);
        let state = ProxyState::from_config(&config);
        group.bench_with_input(
            BenchmarkId::new("upstreams", num_upstreams),
            &num_upstreams,
            |b, _| {
                b.iter(|| state.select_upstream(black_box("domain-0.example.com")));
            },
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_select_upstream_n_routes,
    bench_weighted_round_robin
);
criterion_main!(benches);
