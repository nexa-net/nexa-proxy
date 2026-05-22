window.BENCHMARK_DATA = {
  "lastUpdate": 1779487134321,
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
      }
    ]
  }
}