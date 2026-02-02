# metrics-bridge

<!--
TRANSLATION TEMPLATE
====================
To add a new language:
1. Copy this file and rename to README.{lang_code}.md (e.g., README.ar.md for Arabic)
2. Replace {LANG_NAME} with your language name
3. Update the language selector below (add your language, mark it as **bold**)
4. Translate all content below the badges
5. Keep code blocks, URLs, and technical terms unchanged
6. Add your language to all other README files' language selector
7. Add your language to the main ../README.md language selector (with docs/ prefix)

Language codes: https://en.wikipedia.org/wiki/List_of_ISO_639-1_codes
-->

[English](../README.md) | [Русский](README.ru.md) | [中文](README.zh.md) | [हिंदी](README.hi.md) | [Español](README.es.md) | [Português](README.pt.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Polski](README.pl.md) | [Nederlands](README.nl.md) | [Italiano](README.it.md) | [Türkçe](README.tr.md) | [Українська](README.uk.md) | [Bahasa Indonesia](README.id.md) | [Tiếng Việt](README.vi.md) | [Svenska](README.sv.md) | [Suomi](README.fi.md) | **{LANG_NAME}**

[![Test](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/test.yml/badge.svg)](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/test.yml)
[![Release](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/release.yml/badge.svg)](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/release.yml)
[![codecov](https://codecov.io/gh/brilliant-almazov/metrics-bridge-rs/graph/badge.svg)](https://codecov.io/gh/brilliant-almazov/metrics-bridge-rs)
[![Docker Image Size](https://ghcr-badge.egpl.dev/brilliant-almazov/metrics-bridge-rs/size)](https://github.com/brilliant-almazov/metrics-bridge-rs/pkgs/container/metrics-bridge-rs)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

<!-- TRANSLATE: High-performance Prometheus metrics exporter -->
**{TRANSLATE: High-performance Prometheus metrics exporter}** for [promphp/prometheus_client_php](https://github.com/promphp/prometheus_client_php) Redis storage. Written in Rust for maximum performance and minimal resource consumption.

## {TRANSLATE: Main Features}

- **{TRANSLATE: Ultra fast}** - Sub-millisecond response time (vs 50-200ms via PHP)
- **{TRANSLATE: High throughput}** - 12,500+ requests/second with caching enabled
- **{TRANSLATE: Multiple sources}** - Aggregate metrics from multiple Redis instances
- **{TRANSLATE: Flexible caching}** - Individual cache TTL per source
- **{TRANSLATE: Custom labels}** - Add extra labels to metrics (app, environment, etc.)
- **{TRANSLATE: Security}** - Basic Auth, Bearer Token, IP whitelist, TLS
- **{TRANSLATE: Compression}** - Optional GZIP compression
- **{TRANSLATE: Self-monitoring}** - Built-in metrics of the exporter itself
- **{TRANSLATE: Compact image}** - Docker image <10MB (scratch-based)
- **{TRANSLATE: Compatibility}** - Works with Redis, Dragonfly, Valkey, KeyDB

## {TRANSLATE: Problem}

PHP applications using promphp/prometheus_client_php write metrics to Redis. Reading these metrics through PHP is slow due to framework overhead (~50-200ms with Symfony).

**metrics-bridge** reads metrics directly from Redis and serves them to Prometheus with sub-millisecond latency.

## {TRANSLATE: Performance}

### {TRANSLATE: With cache (recommended)}

| {TRANSLATE: Metric} | {TRANSLATE: Value} |
|---------------------|---------------------|
| {TRANSLATE: Average latency} | **0.8ms** |
| {TRANSLATE: P99 latency} | 7.7ms |
| {TRANSLATE: Throughput} | **12,500+ req/s** |

### {TRANSLATE: Without cache}

| {TRANSLATE: Metric} | {TRANSLATE: Value} |
|---------------------|---------------------|
| {TRANSLATE: Average latency} | ~19ms |
| {TRANSLATE: Throughput} | ~50 req/s |

## {TRANSLATE: Quick Start}

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

## {TRANSLATE: Configuration}

{TRANSLATE: Create} `config.yaml` {TRANSLATE: file}:

```yaml
server:
  port: 9090

  # {TRANSLATE: Optional: GZIP compression}
  gzip_level: 6  # 1-9 (1={TRANSLATE: fastest}, 9={TRANSLATE: best compression})

  auth:
    type: basic  # none | basic | bearer
    username: prometheus
    password: ${METRICS_PASSWORD}  # {TRANSLATE: Environment variable substitution}

  # {TRANSLATE: Optional: IP whitelist}
  allowed_ips:
    - 10.0.0.0/8
    - 192.168.0.0/16

  # {TRANSLATE: Optional: TLS}
  tls:
    cert: /path/to/cert.pem
    key: /path/to/key.pem

sources:
  - name: web-app
    type: promphp-redis
    redis_url: ${REDIS_WEB_URL}
    prefix: "PROMETHEUS_"
    cache_ttl_seconds: 5  # {TRANSLATE: Cache for 5 seconds}
    labels:
      app: web

  - name: worker-app
    type: promphp-redis
    redis_url: ${REDIS_WORKER_URL}
    prefix: "PROMETHEUS_"
    cache_ttl_seconds: 0  # {TRANSLATE: No caching}
    labels:
      app: worker
```

<!-- Continue translating remaining sections... -->

## {TRANSLATE: License}

MIT License - {TRANSLATE: see} [LICENSE](../LICENSE) {TRANSLATE: for details}.

## {TRANSLATE: Contributing}

{TRANSLATE: Welcome! Please read} [CONTRIBUTING.md](../CONTRIBUTING.md) {TRANSLATE: first}.
