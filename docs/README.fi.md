# metrics-bridge

[English](../README.md) | [Русский](README.ru.md) | [中文](README.zh.md) | [Español](README.es.md) | [Português](README.pt.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Polski](README.pl.md) | [Nederlands](README.nl.md) | [Italiano](README.it.md) | [Türkçe](README.tr.md) | [Українська](README.uk.md) | [Svenska](README.sv.md) | **Suomi**

[![Test](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/test.yml/badge.svg)](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/test.yml)
[![Release](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/release.yml/badge.svg)](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/release.yml)
[![codecov](https://codecov.io/gh/brilliant-almazov/metrics-bridge-rs/graph/badge.svg)](https://codecov.io/gh/brilliant-almazov/metrics-bridge-rs)
[![Docker Image Size](https://ghcr-badge.egpl.dev/brilliant-almazov/metrics-bridge-rs/size)](https://github.com/brilliant-almazov/metrics-bridge-rs/pkgs/container/metrics-bridge-rs)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Suorituskykyinen Prometheus-metriikoiden viejä** [promphp/prometheus_client_php](https://github.com/promphp/prometheus_client_php) Redis-tallennukselle. Kirjoitettu Rustilla maksimaalisen suorituskyvyn ja minimaalisen resurssien kulutuksen saavuttamiseksi.

## Pääominaisuudet

- **Erittäin nopea** - Alle millisekunnin vasteaika (vs 50-200ms PHP:n kautta)
- **Korkea läpäisy** - 12 500+ pyyntöä/sekunti välimuistin ollessa päällä
- **Useita lähteitä** - Metriikoiden aggregointi useista Redis-instansseista
- **Joustava välimuisti** - Yksilöllinen välimuistin TTL per lähde
- **Mukautetut tunnisteet** - Lisää ylimääräisiä tunnisteita metriikoihin (app, environment jne.)
- **Turvallisuus** - Basic Auth, Bearer Token, IP-sallittujen lista, TLS
- **Pakkaus** - Valinnainen GZIP-pakkaus
- **Itsevalvonta** - Viejän omat sisäänrakennetut metriikat
- **Kompakti image** - Docker-image <10MB (scratch-pohjainen)
- **Yhteensopivuus** - Toimii Redisin, Dragonfly:n, Valkey:n ja KeyDB:n kanssa

## Ongelma

PHP-sovellukset, jotka käyttävät promphp/prometheus_client_php:ta, kirjoittavat metriikat Redisiin. Näiden metriikoiden lukeminen PHP:n kautta on hidasta kehyksen yleiskustannusten vuoksi (~50-200ms Symfonyn kanssa).

**metrics-bridge** lukee metriikat suoraan Redistä ja tarjoaa ne Prometheukselle alle millisekunnin latenssilla.

## Suorituskyky

### Välimuistilla (suositeltu)

| Metriikka | Arvo |
|-----------|------|
| Keskimääräinen latenssi | **0.8ms** |
| P99 latenssi | 7.7ms |
| Läpäisy | **12 500+ req/s** |

### Ilman välimuistia

| Metriikka | Arvo |
|-----------|------|
| Keskimääräinen latenssi | ~19ms |
| Läpäisy | ~50 req/s |

## Pikaopas

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

## Konfiguraatio

Luo `config.yaml`-tiedosto:

```yaml
server:
  port: 9090

  # Valinnainen: GZIP-pakkaus
  gzip_level: 6  # 1-9 (1=nopein, 9=paras pakkaus)

  auth:
    type: basic  # none | basic | bearer
    username: prometheus
    password: ${METRICS_PASSWORD}  # Ympäristömuuttujien korvaus

  # Valinnainen: IP-sallittujen lista
  allowed_ips:
    - 10.0.0.0/8
    - 192.168.0.0/16

  # Valinnainen: TLS
  tls:
    cert: /path/to/cert.pem
    key: /path/to/key.pem

sources:
  - name: web-app
    type: promphp-redis
    redis_url: ${REDIS_WEB_URL}
    prefix: "PROMETHEUS_"
    cache_ttl_seconds: 5  # Välimuistissa 5 sekuntia
    labels:
      app: web

  - name: worker-app
    type: promphp-redis
    redis_url: ${REDIS_WORKER_URL}
    prefix: "PROMETHEUS_"
    cache_ttl_seconds: 0  # Ei välimuistia
    labels:
      app: worker
```

### Palvelimen parametrit

| Parametri | Oletus | Kuvaus |
|-----------|--------|--------|
| `port` | `9090` | HTTP-palvelimen portti |
| `gzip_level` | - | GZIP-pakkaustaso 1-9, pois käytöstä jos ei asetettu |
| `auth.type` | `none` | Todennustyyppi: `none`, `basic`, `bearer` |
| `auth.username` | - | Käyttäjänimi basic auth -todennukseen |
| `auth.password` | - | Salasana basic auth -todennukseen |
| `auth.token` | - | Token bearer auth -todennukseen |
| `allowed_ips` | `[]` | IP-sallittujen lista (CIDR), tyhjä = salli kaikki |
| `tls.cert` | - | Polku TLS-sertifikaattiin |
| `tls.key` | - | Polku TLS-yksityisavaimeen |

### Lähteiden parametrit

| Parametri | Oletus | Kuvaus |
|-----------|--------|--------|
| `name` | *pakollinen* | Lähteen tunniste |
| `type` | *pakollinen* | Lähteen tyyppi: `promphp-redis` |
| `redis_url` | *pakollinen* | Redis-yhteyden URL |
| `prefix` | `PROMETHEUS_` | Redis-avaimen etuliite |
| `cache_ttl_seconds` | `0` | Välimuistin TTL sekunneissa (0 = pois käytöstä) |
| `label_format` | `auto` | Tunnisteen muoto: `auto`, `json`, `base64` |
| `labels` | `{}` | Ylimääräiset tunnisteet kaikille metriikoille |

## Päätepisteet

| Päätepiste | Todennus | Kuvaus |
|------------|----------|--------|
| `GET /metrics` | Kyllä | Prometheus-metriikat |
| `GET /health` | Ei | Terveystarkistus (aina 200) |
| `GET /ready` | Ei | Valmiustarkistus (200 jos kaikki lähteet terveitä) |

## Omat metriikat

Viejä tarjoaa metriikat itsestään:

```
# Scrape-kesto per lähde
metrics_bridge_scrape_duration_seconds{source="web-app"} 0.003

# Scrape-virheet per lähde
metrics_bridge_scrape_errors_total{source="web-app"} 0

# Lähteen terveys (1=aktiivinen, 0=ei aktiivinen)
metrics_bridge_source_up{source="web-app"} 1

# Konfiguroitujen lähteiden kokonaismäärä
metrics_bridge_sources_total 2

# Viejän tila
metrics_bridge_up 1

# Build-tiedot
metrics_bridge_build_info{version="0.1.0",rust_version="1.84"} 1
```

## Kääntäminen

```bash
# Käännä
cargo build --release

# Testit
cargo test

# Linter
cargo clippy --all-targets -- -D warnings

# Docker-image
docker build -t metrics-bridge .
```

## Lisenssi

MIT License - katso [LICENSE](LICENSE) lisätietoja varten.

## Osallistuminen

Tervetuloa! Lue ensin [CONTRIBUTING.md](CONTRIBUTING.md).
