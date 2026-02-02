# metrics-bridge

[English](../README.md) | [Русский](README.ru.md) | [中文](README.zh.md) | [हिंदी](README.hi.md) | [Español](README.es.md) | [Português](README.pt.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | **Polski** | [Nederlands](README.nl.md) | [Italiano](README.it.md) | [Türkçe](README.tr.md) | [Українська](README.uk.md) | [Bahasa Indonesia](README.id.md) | [Tiếng Việt](README.vi.md) | [Svenska](README.sv.md) | [Suomi](README.fi.md)

[![Test](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/test.yml/badge.svg)](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/test.yml)
[![Release](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/release.yml/badge.svg)](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/release.yml)
[![codecov](https://codecov.io/gh/brilliant-almazov/metrics-bridge-rs/graph/badge.svg)](https://codecov.io/gh/brilliant-almazov/metrics-bridge-rs)
[![Docker Image Size](https://ghcr-badge.egpl.dev/brilliant-almazov/metrics-bridge-rs/size)](https://github.com/brilliant-almazov/metrics-bridge-rs/pkgs/container/metrics-bridge-rs)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Wysokowydajny eksporter metryk Prometheus** dla storage Redis [promphp/prometheus_client_php](https://github.com/promphp/prometheus_client_php). Napisany w Rust dla maksymalnej wydajności i minimalnego zużycia zasobów.

## Główne funkcje

- **Ultraszybki** - Czas odpowiedzi poniżej milisekundy (vs 50-200ms przez PHP)
- **Wysoka przepustowość** - 12 500+ żądań/sekundę z włączonym cache
- **Wiele źródeł** - Agregacja metryk z wielu instancji Redis
- **Elastyczny cache** - Indywidualny TTL cache dla każdego źródła
- **Własne etykiety** - Dodawanie dodatkowych etykiet do metryk (app, environment itd.)
- **Bezpieczeństwo** - Basic Auth, Bearer Token, whitelist IP, TLS
- **Kompresja** - Opcjonalna kompresja GZIP
- **Samomonitorowanie** - Wbudowane metryki samego eksportera
- **Kompaktowy obraz** - Obraz Docker <10MB (oparty na scratch)
- **Kompatybilność** - Działa z Redis, Dragonfly, Valkey, KeyDB

## Problem

Aplikacje PHP używające promphp/prometheus_client_php zapisują metryki w Redis. Odczytywanie tych metryk przez PHP jest wolne z powodu narzutu frameworka (~50-200ms w Symfony).

**metrics-bridge** odczytuje metryki bezpośrednio z Redis i udostępnia je Prometheusowi z opóźnieniem poniżej milisekundy.

## Wydajność

### Z cache (zalecane)

| Metryka | Wartość |
|---------|---------|
| Średnie opóźnienie | **0.8ms** |
| Opóźnienie P99 | 7.7ms |
| Przepustowość | **12 500+ req/s** |

### Bez cache

| Metryka | Wartość |
|---------|---------|
| Średnie opóźnienie | ~19ms |
| Przepustowość | ~50 req/s |

## Szybki start

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

## Konfiguracja

Utwórz plik `config.yaml`:

```yaml
server:
  port: 9090

  # Opcjonalnie: kompresja GZIP
  gzip_level: 6  # 1-9 (1=najszybsza, 9=najlepsza kompresja)

  auth:
    type: basic  # none | basic | bearer
    username: prometheus
    password: ${METRICS_PASSWORD}  # Podstawianie zmiennych środowiskowych

  # Opcjonalnie: whitelist IP
  allowed_ips:
    - 10.0.0.0/8
    - 192.168.0.0/16

  # Opcjonalnie: TLS
  tls:
    cert: /path/to/cert.pem
    key: /path/to/key.pem

sources:
  - name: web-app
    type: promphp-redis
    redis_url: ${REDIS_WEB_URL}
    prefix: "PROMETHEUS_"
    cache_ttl_seconds: 5  # Cache na 5 sekund
    labels:
      app: web

  - name: worker-app
    type: promphp-redis
    redis_url: ${REDIS_WORKER_URL}
    prefix: "PROMETHEUS_"
    cache_ttl_seconds: 0  # Bez cache
    labels:
      app: worker
```

### Parametry serwera

| Parametr | Domyślnie | Opis |
|----------|-----------|------|
| `port` | `9090` | Port serwera HTTP |
| `gzip_level` | - | Poziom kompresji GZIP 1-9, wyłączone jeśli nie ustawione |
| `auth.type` | `none` | Typ uwierzytelniania: `none`, `basic`, `bearer` |
| `auth.username` | - | Nazwa użytkownika dla basic auth |
| `auth.password` | - | Hasło dla basic auth |
| `auth.token` | - | Token dla bearer auth |
| `allowed_ips` | `[]` | Whitelist IP (CIDR), puste = zezwalaj wszystkim |
| `tls.cert` | - | Ścieżka do certyfikatu TLS |
| `tls.key` | - | Ścieżka do klucza prywatnego TLS |

### Parametry źródeł

| Parametr | Domyślnie | Opis |
|----------|-----------|------|
| `name` | *wymagane* | Identyfikator źródła |
| `type` | *wymagane* | Typ źródła: `promphp-redis` |
| `redis_url` | *wymagane* | URL połączenia Redis |
| `prefix` | `PROMETHEUS_` | Prefiks kluczy Redis |
| `cache_ttl_seconds` | `0` | TTL cache w sekundach (0 = wyłączone) |
| `label_format` | `auto` | Format etykiet: `auto`, `json`, `base64` |
| `labels` | `{}` | Dodatkowe etykiety dla wszystkich metryk |

## Endpointy

| Endpoint | Uwierzytelnianie | Opis |
|----------|------------------|------|
| `GET /metrics` | Tak | Metryki Prometheus |
| `GET /health` | Nie | Sprawdzenie stanu (zawsze 200) |
| `GET /ready` | Nie | Sprawdzenie gotowości (200 jeśli wszystkie źródła zdrowe) |

## Własne metryki

Eksporter udostępnia metryki o sobie:

```
# Czas scrape per źródło
metrics_bridge_scrape_duration_seconds{source="web-app"} 0.003

# Błędy scrape per źródło
metrics_bridge_scrape_errors_total{source="web-app"} 0

# Zdrowie źródła (1=działa, 0=nie działa)
metrics_bridge_source_up{source="web-app"} 1

# Całkowita liczba skonfigurowanych źródeł
metrics_bridge_sources_total 2

# Status eksportera
metrics_bridge_up 1

# Informacje o buildzie
metrics_bridge_build_info{version="0.1.0",rust_version="1.84"} 1
```

## Budowanie

```bash
# Budowanie
cargo build --release

# Testy
cargo test

# Linter
cargo clippy --all-targets -- -D warnings

# Obraz Docker
docker build -t metrics-bridge .
```

## Licencja

MIT License - szczegóły w [LICENSE](LICENSE).

## Wkład

Mile widziany! Proszę najpierw przeczytać [CONTRIBUTING.md](CONTRIBUTING.md).
