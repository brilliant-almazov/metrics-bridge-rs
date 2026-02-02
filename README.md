# metrics-bridge

[![Test](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/test.yml/badge.svg)](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/test.yml)
[![Release](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/release.yml/badge.svg)](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/release.yml)
[![codecov](https://codecov.io/gh/brilliant-almazov/metrics-bridge-rs/graph/badge.svg)](https://codecov.io/gh/brilliant-almazov/metrics-bridge-rs)
[![Docker Image Size](https://ghcr-badge.egpl.dev/brilliant-almazov/metrics-bridge-rs/size)](https://github.com/brilliant-almazov/metrics-bridge-rs/pkgs/container/metrics-bridge-rs)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

### Performance (auto-updated from CI)

| Metric | With Cache | Without Cache |
|--------|------------|---------------|
| RPS | ![RPS Cached](https://img.shields.io/endpoint?url=https://gist.githubusercontent.com/brilliant-almazov/0ef94616a74c387a9626f71245f9533c/raw/rps-cached.json) | ![RPS Uncached](https://img.shields.io/endpoint?url=https://gist.githubusercontent.com/brilliant-almazov/0ef94616a74c387a9626f71245f9533c/raw/rps-uncached.json) |
| Latency | ![Latency Cached](https://img.shields.io/endpoint?url=https://gist.githubusercontent.com/brilliant-almazov/0ef94616a74c387a9626f71245f9533c/raw/latency-cached.json) | ![Latency Uncached](https://img.shields.io/endpoint?url=https://gist.githubusercontent.com/brilliant-almazov/0ef94616a74c387a9626f71245f9533c/raw/latency-uncached.json) |

### Tested Redis-Compatible Stores

| Store | Status |
|-------|--------|
| Redis 7 | ![Redis 7](https://img.shields.io/badge/Redis%207-tested-success) |
| Redis 6 | ![Redis 6](https://img.shields.io/badge/Redis%206-tested-success) |
| Dragonfly | ![Dragonfly](https://img.shields.io/badge/Dragonfly-tested-success) |
| Valkey | ![Valkey](https://img.shields.io/badge/Valkey-tested-success) |
| KeyDB | ![KeyDB](https://img.shields.io/badge/KeyDB-tested-success) |

Fast Prometheus metrics exporter for [promphp/prometheus_client_php](https://github.com/promphp/prometheus_client_php) Redis storage.

## Problem

PHP applications using promphp/prometheus_client_php write metrics to Redis. Reading these metrics through PHP is slow due to framework overhead (~50-200ms with Symfony).

**metrics-bridge** reads metrics directly from Redis and serves them to Prometheus with sub-millisecond latency.

## Features

- 🚀 **Fast** - Sub-millisecond response times (vs 50-200ms through PHP)
- ⚡ **Per-source caching** - Individual cache TTL per source, 12,500+ RPS with cache
- 🔌 **Multiple sources** - Aggregate metrics from multiple Redis instances
- 🏷️ **Extra labels** - Add custom labels per source (e.g., `app`, `environment`)
- 🔒 **Auth support** - Basic auth, Bearer token, IP whitelist
- 📊 **Self-metrics** - Built-in metrics for monitoring the exporter itself
- 🐳 **Tiny image** - <10MB Docker image (scratch-based)
- ✅ **Tested** - Works with Redis, Dragonfly, Valkey, KeyDB

## Quick Start

### Docker

```bash
docker run -p 9090:9090 \
  -v $(pwd)/config.yaml:/config.yaml \
  ghcr.io/brilliant-almazov/metrics-bridge-rs:latest
```

### Docker Compose

```yaml
services:
  metrics-bridge:
    image: ghcr.io/brilliant-almazov/metrics-bridge-rs:latest
    ports:
      - "9090:9090"
    volumes:
      - ./config.yaml:/config.yaml
    environment:
      - REDIS_URL=redis://redis:6379
```

## Configuration

Create `config.yaml`:

```yaml
server:
  port: 9090

  auth:
    type: basic  # none | basic | bearer
    username: prometheus
    password: ${METRICS_PASSWORD}  # Environment variable substitution

  # Optional: IP whitelist
  allowed_ips:
    - 10.0.0.0/8
    - 192.168.0.0/16

  # Optional: TLS
  tls:
    cert: /path/to/cert.pem
    key: /path/to/key.pem

sources:
  - name: web-app
    type: promphp-redis
    redis_url: ${REDIS_WEB_URL}
    prefix: "PROMETHEUS_"
    cache_ttl_seconds: 5  # Cache for 5 seconds
    labels:
      app: web

  - name: worker-app
    type: promphp-redis
    redis_url: ${REDIS_WORKER_URL}
    prefix: "PROMETHEUS_"
    cache_ttl_seconds: 0  # No caching
    labels:
      app: worker
```

### Configuration Options

#### Server

| Option | Default | Description |
|--------|---------|-------------|
| `port` | `9090` | HTTP server port |
| `auth.type` | `none` | Authentication type: `none`, `basic`, `bearer` |
| `auth.username` | - | Username for basic auth |
| `auth.password` | - | Password for basic auth |
| `auth.token` | - | Token for bearer auth |
| `allowed_ips` | `[]` | IP whitelist (CIDR notation), empty = allow all |
| `tls.cert` | - | Path to TLS certificate |
| `tls.key` | - | Path to TLS private key |

#### Sources

| Option | Default | Description |
|--------|---------|-------------|
| `name` | *required* | Source identifier |
| `type` | *required* | Source type: `promphp-redis` |
| `redis_url` | *required* | Redis connection URL |
| `prefix` | `PROMETHEUS_` | Redis key prefix |
| `cache_ttl_seconds` | `0` | Cache TTL in seconds (0 = disabled) |
| `labels` | `{}` | Extra labels to add to all metrics |

### Environment Variables

- `CONFIG_FILE` - Path to config file (default: `config.yaml`)
- `CONFIG_BASE64` - Base64-encoded config (for Docker/K8s secrets)
- `RUST_LOG` - Log level (default: `metrics_bridge=info`)

## Endpoints

| Endpoint | Auth | Description |
|----------|------|-------------|
| `GET /metrics` | Yes | Prometheus metrics |
| `GET /health` | No | Health check (always 200) |
| `GET /ready` | No | Readiness check (200 if all sources healthy) |

## Self-Metrics

The exporter exposes its own metrics:

```
# Scrape duration per source
metrics_bridge_scrape_duration_seconds{source="web-app"} 0.003

# Scrape errors per source
metrics_bridge_scrape_errors_total{source="web-app"} 0

# Source health (1=up, 0=down)
metrics_bridge_source_up{source="web-app"} 1

# Total configured sources
metrics_bridge_sources_total 2

# Exporter status
metrics_bridge_up 1

# Build info
metrics_bridge_build_info{version="0.1.0",rust_version="1.84"} 1
```

## promphp Redis Format

This exporter reads metrics stored by [promphp/prometheus_client_php](https://github.com/promphp/prometheus_client_php) in Redis:

- `{prefix}:COUNTER_METRIC_KEYS` - SET with counter key names
- `{prefix}:GAUGE_METRIC_KEYS` - SET with gauge key names
- `{prefix}:HISTOGRAM_METRIC_KEYS` - SET with histogram key names
- `{prefix}:{type}:{name}` - HASH with:
  - `__meta` - JSON metadata: `{"name", "help", "type", "labelNames", "buckets"}`
  - `base64(json(labelValues))` → value

## Building

```bash
# Build
cargo build --release

# Run tests
cargo test

# Run clippy
cargo clippy --all-targets -- -D warnings

# Build Docker image
docker build -t metrics-bridge .
```

## Testing with Multiple Redis Stores

Run E2E tests against Redis, Dragonfly, Valkey, and KeyDB:

```bash
docker compose -f docker-compose.test.yml up --build
```

## Performance

### With Per-Source Caching (Recommended)

Each source can have its own `cache_ttl_seconds`. Example with 5 second cache:

| Metric | Value |
|--------|-------|
| Avg Latency | **0.8ms** |
| P99 Latency | 7.7ms |
| Throughput | **12,500+ req/s** |

### Without Caching

Direct Redis fetch on every request (`cache_ttl_seconds: 0`):

| Metric | Value |
|--------|-------|
| Avg Latency | ~19ms |
| Throughput | ~50 req/s |

### CI E2E Results (no cache)

| Store | Avg Latency | Throughput |
|-------|-------------|------------|
| Redis 7 | ~5ms | ~168 req/s |
| Redis 6 | ~5ms | ~189 req/s |
| Dragonfly | ~6ms | ~157 req/s |
| Valkey | ~6ms | ~166 req/s |
| KeyDB | ~6ms | ~164 req/s |

> **Recommendation**: Enable per-source caching with `cache_ttl_seconds: 5` for production.
> Prometheus default scrape interval is 15s, so 5s cache is safe. Each source can have different TTL.

## License

MIT License - see [LICENSE](LICENSE) for details.

## Contributing

Contributions welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) first.
