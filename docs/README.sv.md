# metrics-bridge

[English](../README.md) | [Русский](README.ru.md) | [中文](README.zh.md) | [हिंदी](README.hi.md) | [Español](README.es.md) | [Português](README.pt.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Polski](README.pl.md) | [Nederlands](README.nl.md) | [Italiano](README.it.md) | [Türkçe](README.tr.md) | [Українська](README.uk.md) | [Bahasa Indonesia](README.id.md) | [Tiếng Việt](README.vi.md) | **Svenska** | [Suomi](README.fi.md)

[![Test](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/test.yml/badge.svg)](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/test.yml)
[![Release](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/release.yml/badge.svg)](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/release.yml)
[![codecov](https://codecov.io/gh/brilliant-almazov/metrics-bridge-rs/graph/badge.svg)](https://codecov.io/gh/brilliant-almazov/metrics-bridge-rs)
[![Docker Image Size](https://ghcr-badge.egpl.dev/brilliant-almazov/metrics-bridge-rs/size)](https://github.com/brilliant-almazov/metrics-bridge-rs/pkgs/container/metrics-bridge-rs)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Högpresterande Prometheus-metrikexportör** för [promphp/prometheus_client_php](https://github.com/promphp/prometheus_client_php) Redis-lagring. Skriven i Rust för maximal prestanda och minimal resursförbrukning.

## Huvudfunktioner

- **Ultrasnabb** - Svarstid under millisekund (vs 50-200ms via PHP)
- **Hög genomströmning** - 12 500+ förfrågningar/sekund med cache aktiverad
- **Flera källor** - Aggregera metriker från flera Redis-instanser
- **Flexibel caching** - Individuell cache-TTL per källa
- **Anpassade etiketter** - Lägg till extra etiketter till metriker (app, environment, etc.)
- **Säkerhet** - Basic Auth, Bearer Token, IP-vitlista, TLS
- **Komprimering** - Valfri GZIP-komprimering
- **Självövervakning** - Inbyggda metriker för exportören själv
- **Kompakt image** - Docker-image <10MB (scratch-baserad)
- **Kompatibilitet** - Fungerar med Redis, Dragonfly, Valkey, KeyDB

## Problem

PHP-applikationer som använder promphp/prometheus_client_php skriver metriker till Redis. Att läsa dessa metriker via PHP är långsamt på grund av framework-overhead (~50-200ms med Symfony).

**metrics-bridge** läser metriker direkt från Redis och levererar dem till Prometheus med latens under millisekund.

## Prestanda

### Med cache (rekommenderat)

| Metrik | Värde |
|--------|-------|
| Genomsnittlig latens | **0.8ms** |
| P99 latens | 7.7ms |
| Genomströmning | **12 500+ req/s** |

### Utan cache

| Metrik | Värde |
|--------|-------|
| Genomsnittlig latens | ~19ms |
| Genomströmning | ~50 req/s |

## Snabbstart

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

## Konfiguration

Skapa filen `config.yaml`:

```yaml
server:
  port: 9090

  # Valfritt: GZIP-komprimering
  gzip_level: 6  # 1-9 (1=snabbast, 9=bäst komprimering)

  auth:
    type: basic  # none | basic | bearer
    username: prometheus
    password: ${METRICS_PASSWORD}  # Miljövariabelsubstitution

  # Valfritt: IP-vitlista
  allowed_ips:
    - 10.0.0.0/8
    - 192.168.0.0/16

  # Valfritt: TLS
  tls:
    cert: /path/to/cert.pem
    key: /path/to/key.pem

sources:
  - name: web-app
    type: promphp-redis
    redis_url: ${REDIS_WEB_URL}
    prefix: "PROMETHEUS_"
    cache_ttl_seconds: 5  # Cacha i 5 sekunder
    labels:
      app: web

  - name: worker-app
    type: promphp-redis
    redis_url: ${REDIS_WORKER_URL}
    prefix: "PROMETHEUS_"
    cache_ttl_seconds: 0  # Ingen caching
    labels:
      app: worker
```

### Serverparametrar

| Parameter | Standard | Beskrivning |
|-----------|----------|-------------|
| `port` | `9090` | HTTP-serverport |
| `gzip_level` | - | GZIP-komprimeringsnivå 1-9, inaktiverad om ej angiven |
| `auth.type` | `none` | Autentiseringstyp: `none`, `basic`, `bearer` |
| `auth.username` | - | Användarnamn för basic auth |
| `auth.password` | - | Lösenord för basic auth |
| `auth.token` | - | Token för bearer auth |
| `allowed_ips` | `[]` | IP-vitlista (CIDR), tom = tillåt alla |
| `tls.cert` | - | Sökväg till TLS-certifikat |
| `tls.key` | - | Sökväg till TLS-privat nyckel |

### Källparametrar

| Parameter | Standard | Beskrivning |
|-----------|----------|-------------|
| `name` | *obligatoriskt* | Källidentifierare |
| `type` | *obligatoriskt* | Källtyp: `promphp-redis` |
| `redis_url` | *obligatoriskt* | Redis-anslutnings-URL |
| `prefix` | `PROMETHEUS_` | Redis-nyckelprefix |
| `cache_ttl_seconds` | `0` | Cache-TTL i sekunder (0 = inaktiverad) |
| `label_format` | `auto` | Etikettformat: `auto`, `json`, `base64` |
| `labels` | `{}` | Extra etiketter för alla metriker |

## Endpoints

| Endpoint | Autentisering | Beskrivning |
|----------|---------------|-------------|
| `GET /metrics` | Ja | Prometheus-metriker |
| `GET /health` | Nej | Hälsokontroll (alltid 200) |
| `GET /ready` | Nej | Beredskapskontroll (200 om alla källor är friska) |

## Egna metriker

Exportören exponerar metriker om sig själv:

```
# Scrape-varaktighet per källa
metrics_bridge_scrape_duration_seconds{source="web-app"} 0.003

# Scrape-fel per källa
metrics_bridge_scrape_errors_total{source="web-app"} 0

# Källhälsa (1=aktiv, 0=inaktiv)
metrics_bridge_source_up{source="web-app"} 1

# Totalt antal konfigurerade källor
metrics_bridge_sources_total 2

# Exportörstatus
metrics_bridge_up 1

# Build-information
metrics_bridge_build_info{version="0.1.0",rust_version="1.84"} 1
```

## Byggning

```bash
# Bygg
cargo build --release

# Tester
cargo test

# Linter
cargo clippy --all-targets -- -D warnings

# Docker-image
docker build -t metrics-bridge .
```

## Licens

MIT License - se [LICENSE](LICENSE) för detaljer.

## Bidrag

Välkommen! Läs först [CONTRIBUTING.md](CONTRIBUTING.md).
