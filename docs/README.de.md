# metrics-bridge

[English](../README.md) | [Русский](README.ru.md) | [中文](README.zh.md) | [हिंदी](README.hi.md) | [Español](README.es.md) | [Português](README.pt.md) | [Français](README.fr.md) | **Deutsch** | [日本語](README.ja.md) | [한국어](README.ko.md) | [Polski](README.pl.md) | [Nederlands](README.nl.md) | [Italiano](README.it.md) | [Türkçe](README.tr.md) | [Українська](README.uk.md) | [Bahasa Indonesia](README.id.md) | [Tiếng Việt](README.vi.md) | [Svenska](README.sv.md) | [Suomi](README.fi.md)

[![Test](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/test.yml/badge.svg)](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/test.yml)
[![Release](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/release.yml/badge.svg)](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/release.yml)
[![codecov](https://codecov.io/gh/brilliant-almazov/metrics-bridge-rs/graph/badge.svg)](https://codecov.io/gh/brilliant-almazov/metrics-bridge-rs)
[![Docker Image Size](https://ghcr-badge.egpl.dev/brilliant-almazov/metrics-bridge-rs/size)](https://github.com/brilliant-almazov/metrics-bridge-rs/pkgs/container/metrics-bridge-rs)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Hochleistungs-Prometheus-Metrik-Exporter** für [promphp/prometheus_client_php](https://github.com/promphp/prometheus_client_php) Redis-Speicher. In Rust geschrieben für maximale Leistung und minimalen Ressourcenverbrauch.

## Hauptfunktionen

- **Ultraschnell** - Sub-Millisekunden-Antwortzeit (vs. 50-200ms über PHP)
- **Hoher Durchsatz** - 12.500+ Anfragen/Sekunde mit aktiviertem Cache
- **Mehrere Quellen** - Aggregation von Metriken aus mehreren Redis-Instanzen
- **Flexibles Caching** - Individuelles Cache-TTL pro Quelle
- **Benutzerdefinierte Labels** - Fügen Sie zusätzliche Labels zu Metriken hinzu (app, environment, etc.)
- **Sicherheit** - Basic Auth, Bearer Token, IP-Whitelist, TLS
- **Komprimierung** - Optionale GZIP-Komprimierung
- **Selbstüberwachung** - Integrierte Metriken des Exporters selbst
- **Kompaktes Image** - Docker-Image <10MB (scratch-basiert)
- **Kompatibilität** - Funktioniert mit Redis, Dragonfly, Valkey, KeyDB

## Problem

PHP-Anwendungen, die promphp/prometheus_client_php verwenden, schreiben Metriken in Redis. Das Lesen dieser Metriken über PHP ist aufgrund des Framework-Overheads langsam (~50-200ms mit Symfony).

**metrics-bridge** liest Metriken direkt aus Redis und liefert sie an Prometheus mit Sub-Millisekunden-Latenz.

## Leistung

### Mit Cache (empfohlen)

| Metrik | Wert |
|--------|------|
| Durchschnittliche Latenz | **0.8ms** |
| P99 Latenz | 7.7ms |
| Durchsatz | **12.500+ req/s** |

### Ohne Cache

| Metrik | Wert |
|--------|------|
| Durchschnittliche Latenz | ~19ms |
| Durchsatz | ~50 req/s |

## Schnellstart

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

Erstellen Sie die Datei `config.yaml`:

```yaml
server:
  port: 9090

  # Optional: GZIP-Komprimierung
  gzip_level: 6  # 1-9 (1=schnellste, 9=beste Komprimierung)

  auth:
    type: basic  # none | basic | bearer
    username: prometheus
    password: ${METRICS_PASSWORD}  # Umgebungsvariablen-Ersetzung

  # Optional: IP-Whitelist
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
    cache_ttl_seconds: 5  # 5 Sekunden cachen
    labels:
      app: web

  - name: worker-app
    type: promphp-redis
    redis_url: ${REDIS_WORKER_URL}
    prefix: "PROMETHEUS_"
    cache_ttl_seconds: 0  # Kein Caching
    labels:
      app: worker
```

### Server-Parameter

| Parameter | Standard | Beschreibung |
|-----------|----------|--------------|
| `port` | `9090` | HTTP-Server-Port |
| `gzip_level` | - | GZIP-Komprimierungsstufe 1-9, deaktiviert wenn nicht gesetzt |
| `auth.type` | `none` | Authentifizierungstyp: `none`, `basic`, `bearer` |
| `auth.username` | - | Benutzername für Basic Auth |
| `auth.password` | - | Passwort für Basic Auth |
| `auth.token` | - | Token für Bearer Auth |
| `allowed_ips` | `[]` | IP-Whitelist (CIDR), leer = alle erlauben |
| `tls.cert` | - | Pfad zum TLS-Zertifikat |
| `tls.key` | - | Pfad zum privaten TLS-Schlüssel |

### Quellen-Parameter

| Parameter | Standard | Beschreibung |
|-----------|----------|--------------|
| `name` | *erforderlich* | Quellenidentifikator |
| `type` | *erforderlich* | Quellentyp: `promphp-redis` |
| `redis_url` | *erforderlich* | Redis-Verbindungs-URL |
| `prefix` | `PROMETHEUS_` | Redis-Schlüssel-Präfix |
| `cache_ttl_seconds` | `0` | Cache-TTL in Sekunden (0 = deaktiviert) |
| `label_format` | `auto` | Label-Format: `auto`, `json`, `base64` |
| `labels` | `{}` | Zusätzliche Labels für alle Metriken |

## Endpunkte

| Endpunkt | Authentifizierung | Beschreibung |
|----------|-------------------|--------------|
| `GET /metrics` | Ja | Prometheus-Metriken |
| `GET /health` | Nein | Gesundheitsprüfung (immer 200) |
| `GET /ready` | Nein | Bereitschaftsprüfung (200 wenn alle Quellen gesund) |

## Eigene Metriken

Der Exporter stellt Metriken über sich selbst bereit:

```
# Scrape-Dauer pro Quelle
metrics_bridge_scrape_duration_seconds{source="web-app"} 0.003

# Scrape-Fehler pro Quelle
metrics_bridge_scrape_errors_total{source="web-app"} 0

# Quellen-Gesundheit (1=aktiv, 0=inaktiv)
metrics_bridge_source_up{source="web-app"} 1

# Gesamtzahl konfigurierter Quellen
metrics_bridge_sources_total 2

# Exporter-Status
metrics_bridge_up 1

# Build-Informationen
metrics_bridge_build_info{version="0.1.0",rust_version="1.84"} 1
```

## Kompilierung

```bash
# Kompilieren
cargo build --release

# Tests
cargo test

# Linter
cargo clippy --all-targets -- -D warnings

# Docker-Image
docker build -t metrics-bridge .
```

## Lizenz

MIT License - siehe [LICENSE](LICENSE) für Details.

## Beiträge

Willkommen! Bitte lesen Sie zuerst [CONTRIBUTING.md](CONTRIBUTING.md).
