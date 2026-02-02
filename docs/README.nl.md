# metrics-bridge

[English](../README.md) | [Русский](README.ru.md) | [中文](README.zh.md) | [हिंदी](README.hi.md) | [Español](README.es.md) | [Português](README.pt.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Polski](README.pl.md) | **Nederlands** | [Italiano](README.it.md) | [Türkçe](README.tr.md) | [Українська](README.uk.md) | [Bahasa Indonesia](README.id.md) | [Tiếng Việt](README.vi.md) | [Svenska](README.sv.md) | [Suomi](README.fi.md)

[![Test](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/test.yml/badge.svg)](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/test.yml)
[![Release](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/release.yml/badge.svg)](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/release.yml)
[![codecov](https://codecov.io/gh/brilliant-almazov/metrics-bridge-rs/graph/badge.svg)](https://codecov.io/gh/brilliant-almazov/metrics-bridge-rs)
[![Docker Image Size](https://ghcr-badge.egpl.dev/brilliant-almazov/metrics-bridge-rs/size)](https://github.com/brilliant-almazov/metrics-bridge-rs/pkgs/container/metrics-bridge-rs)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Hoogwaardige Prometheus metrics exporter** voor [promphp/prometheus_client_php](https://github.com/promphp/prometheus_client_php) Redis-opslag. Geschreven in Rust voor maximale prestaties en minimaal resourcegebruik.

## Belangrijkste functies

- **Ultrasnel** - Sub-milliseconde responstijd (vs 50-200ms via PHP)
- **Hoge doorvoer** - 12.500+ verzoeken/seconde met cache ingeschakeld
- **Meerdere bronnen** - Aggregeer metrics van meerdere Redis-instanties
- **Flexibele caching** - Individuele cache TTL per bron
- **Aangepaste labels** - Voeg extra labels toe aan metrics (app, environment, etc.)
- **Beveiliging** - Basic Auth, Bearer Token, IP-whitelist, TLS
- **Compressie** - Optionele GZIP-compressie
- **Zelfmonitoring** - Ingebouwde metrics van de exporter zelf
- **Compact image** - Docker-image <10MB (scratch-gebaseerd)
- **Compatibiliteit** - Werkt met Redis, Dragonfly, Valkey, KeyDB

## Probleem

PHP-applicaties die promphp/prometheus_client_php gebruiken, schrijven metrics naar Redis. Het lezen van deze metrics via PHP is traag door framework-overhead (~50-200ms met Symfony).

**metrics-bridge** leest metrics rechtstreeks uit Redis en levert ze aan Prometheus met sub-milliseconde latentie.

## Prestaties

### Met cache (aanbevolen)

| Metric | Waarde |
|--------|--------|
| Gemiddelde latentie | **0.8ms** |
| P99 latentie | 7.7ms |
| Doorvoer | **12.500+ req/s** |

### Zonder cache

| Metric | Waarde |
|--------|--------|
| Gemiddelde latentie | ~19ms |
| Doorvoer | ~50 req/s |

## Snelle start

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

## Configuratie

Maak het bestand `config.yaml`:

```yaml
server:
  port: 9090

  # Optioneel: GZIP-compressie
  gzip_level: 6  # 1-9 (1=snelste, 9=beste compressie)

  auth:
    type: basic  # none | basic | bearer
    username: prometheus
    password: ${METRICS_PASSWORD}  # Omgevingsvariabele substitutie

  # Optioneel: IP-whitelist
  allowed_ips:
    - 10.0.0.0/8
    - 192.168.0.0/16

  # Optioneel: TLS
  tls:
    cert: /path/to/cert.pem
    key: /path/to/key.pem

sources:
  - name: web-app
    type: promphp-redis
    redis_url: ${REDIS_WEB_URL}
    prefix: "PROMETHEUS_"
    cache_ttl_seconds: 5  # 5 seconden cachen
    labels:
      app: web

  - name: worker-app
    type: promphp-redis
    redis_url: ${REDIS_WORKER_URL}
    prefix: "PROMETHEUS_"
    cache_ttl_seconds: 0  # Geen caching
    labels:
      app: worker
```

### Server parameters

| Parameter | Standaard | Beschrijving |
|-----------|-----------|--------------|
| `port` | `9090` | HTTP-serverpoort |
| `gzip_level` | - | GZIP-compressieniveau 1-9, uitgeschakeld indien niet ingesteld |
| `auth.type` | `none` | Authenticatietype: `none`, `basic`, `bearer` |
| `auth.username` | - | Gebruikersnaam voor basic auth |
| `auth.password` | - | Wachtwoord voor basic auth |
| `auth.token` | - | Token voor bearer auth |
| `allowed_ips` | `[]` | IP-whitelist (CIDR), leeg = alles toestaan |
| `tls.cert` | - | Pad naar TLS-certificaat |
| `tls.key` | - | Pad naar TLS-privésleutel |

### Bron parameters

| Parameter | Standaard | Beschrijving |
|-----------|-----------|--------------|
| `name` | *vereist* | Bronidentificator |
| `type` | *vereist* | Brontype: `promphp-redis` |
| `redis_url` | *vereist* | Redis-verbindings-URL |
| `prefix` | `PROMETHEUS_` | Redis-sleutelprefix |
| `cache_ttl_seconds` | `0` | Cache TTL in seconden (0 = uitgeschakeld) |
| `label_format` | `auto` | Labelformaat: `auto`, `json`, `base64` |
| `labels` | `{}` | Extra labels voor alle metrics |

## Endpoints

| Endpoint | Authenticatie | Beschrijving |
|----------|---------------|--------------|
| `GET /metrics` | Ja | Prometheus metrics |
| `GET /health` | Nee | Gezondheidscontrole (altijd 200) |
| `GET /ready` | Nee | Gereedheidscontrole (200 als alle bronnen gezond) |

## Eigen metrics

De exporter biedt metrics over zichzelf:

```
# Scrape-duur per bron
metrics_bridge_scrape_duration_seconds{source="web-app"} 0.003

# Scrape-fouten per bron
metrics_bridge_scrape_errors_total{source="web-app"} 0

# Brongezondheid (1=actief, 0=inactief)
metrics_bridge_source_up{source="web-app"} 1

# Totaal aantal geconfigureerde bronnen
metrics_bridge_sources_total 2

# Exporter-status
metrics_bridge_up 1

# Build-informatie
metrics_bridge_build_info{version="0.1.0",rust_version="1.84"} 1
```

## Bouwen

```bash
# Bouwen
cargo build --release

# Testen
cargo test

# Linter
cargo clippy --all-targets -- -D warnings

# Docker-image
docker build -t metrics-bridge .
```

## Licentie

MIT License - zie [LICENSE](LICENSE) voor details.

## Bijdragen

Welkom! Lees eerst [CONTRIBUTING.md](CONTRIBUTING.md).
