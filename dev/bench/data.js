window.BENCHMARK_DATA = {
  "lastUpdate": 1779487266696,
  "repoUrl": "https://github.com/nexa-net/nexa-proxy",
  "entries": {
    "Benchmark": [
      {
        "commit": {
          "author": {
            "email": "nassime.abdiou@icloud.com",
            "name": "Nassime Abdiou",
            "username": "na2sime"
          },
          "committer": {
            "email": "nassime.abdiou@icloud.com",
            "name": "Nassime Abdiou",
            "username": "na2sime"
          },
          "distinct": true,
          "id": "f0c2541707b353c6837667a1288f2fc8d2ea2651",
          "message": "style: fix formatting in proxy tests and benchmarks",
          "timestamp": "2026-05-22T23:53:35+02:00",
          "tree_id": "348394be3b6dc91fcf77d1edb9b5fac4ee91bc78",
          "url": "https://github.com/nexa-net/nexa-proxy/commit/f0c2541707b353c6837667a1288f2fc8d2ea2651"
        },
        "date": 1779487134052,
        "tool": "cargo",
        "benches": [
          {
            "name": "select_upstream/routes/10",
            "value": 30,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "select_upstream/routes/100",
            "value": 30,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "select_upstream/routes/1000",
            "value": 30,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "weighted_round_robin/upstreams/3",
            "value": 30,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "weighted_round_robin/upstreams/10",
            "value": 36,
            "range": "± 0",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "nassime.abdiou@icloud.com",
            "name": "Nassime Abdiou",
            "username": "na2sime"
          },
          "committer": {
            "email": "nassime.abdiou@icloud.com",
            "name": "Nassime Abdiou",
            "username": "na2sime"
          },
          "distinct": true,
          "id": "f678680cf4812d99a2cd6b5f76c1a154aeffa387",
          "message": "ci: add benchmark workflow with regression detection",
          "timestamp": "2026-05-22T21:54:27+02:00",
          "tree_id": "e6579b161aed9a15885c93a97ca7cf4ec0aa9002",
          "url": "https://github.com/nexa-net/nexa-proxy/commit/f678680cf4812d99a2cd6b5f76c1a154aeffa387"
        },
        "date": 1779487265779,
        "tool": "cargo",
        "benches": [
          {
            "name": "select_upstream/routes/10",
            "value": 30,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "select_upstream/routes/100",
            "value": 30,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "select_upstream/routes/1000",
            "value": 30,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "weighted_round_robin/upstreams/3",
            "value": 30,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "weighted_round_robin/upstreams/10",
            "value": 34,
            "range": "± 0",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}