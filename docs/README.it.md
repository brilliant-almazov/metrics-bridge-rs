# metrics-bridge

[English](../README.md) | [Русский](README.ru.md) | [中文](README.zh.md) | [Español](README.es.md) | [Português](README.pt.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Polski](README.pl.md) | [Nederlands](README.nl.md) | **Italiano** | [Türkçe](README.tr.md) | [Українська](README.uk.md)

[![Test](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/test.yml/badge.svg)](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/test.yml)
[![Release](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/release.yml/badge.svg)](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/release.yml)
[![codecov](https://codecov.io/gh/brilliant-almazov/metrics-bridge-rs/graph/badge.svg)](https://codecov.io/gh/brilliant-almazov/metrics-bridge-rs)
[![Docker Image Size](https://ghcr-badge.egpl.dev/brilliant-almazov/metrics-bridge-rs/size)](https://github.com/brilliant-almazov/metrics-bridge-rs/pkgs/container/metrics-bridge-rs)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Esportatore di metriche Prometheus ad alte prestazioni** per lo storage Redis di [promphp/prometheus_client_php](https://github.com/promphp/prometheus_client_php). Scritto in Rust per massime prestazioni e minimo consumo di risorse.

## Funzionalità principali

- **Ultra veloce** - Tempo di risposta sotto il millisecondo (vs 50-200ms tramite PHP)
- **Alto throughput** - 12.500+ richieste/secondo con cache abilitata
- **Sorgenti multiple** - Aggregazione di metriche da più istanze Redis
- **Cache flessibile** - TTL cache individuale per ogni sorgente
- **Etichette personalizzate** - Aggiunta di etichette extra alle metriche (app, environment, ecc.)
- **Sicurezza** - Basic Auth, Bearer Token, whitelist IP, TLS
- **Compressione** - Compressione GZIP opzionale
- **Auto-monitoraggio** - Metriche integrate dell'esportatore stesso
- **Immagine compatta** - Immagine Docker <10MB (basata su scratch)
- **Compatibilità** - Funziona con Redis, Dragonfly, Valkey, KeyDB

## Problema

Le applicazioni PHP che usano promphp/prometheus_client_php scrivono metriche in Redis. La lettura di queste metriche tramite PHP è lenta a causa dell'overhead del framework (~50-200ms con Symfony).

**metrics-bridge** legge le metriche direttamente da Redis e le serve a Prometheus con latenza sotto il millisecondo.

## Prestazioni

### Con cache (consigliato)

| Metrica | Valore |
|---------|--------|
| Latenza media | **0.8ms** |
| Latenza P99 | 7.7ms |
| Throughput | **12.500+ req/s** |

### Senza cache

| Metrica | Valore |
|---------|--------|
| Latenza media | ~19ms |
| Throughput | ~50 req/s |

## Avvio rapido

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

## Configurazione

Crea il file `config.yaml`:

```yaml
server:
  port: 9090

  # Opzionale: compressione GZIP
  gzip_level: 6  # 1-9 (1=più veloce, 9=migliore compressione)

  auth:
    type: basic  # none | basic | bearer
    username: prometheus
    password: ${METRICS_PASSWORD}  # Sostituzione variabili d'ambiente

  # Opzionale: whitelist IP
  allowed_ips:
    - 10.0.0.0/8
    - 192.168.0.0/16

  # Opzionale: TLS
  tls:
    cert: /path/to/cert.pem
    key: /path/to/key.pem

sources:
  - name: web-app
    type: promphp-redis
    redis_url: ${REDIS_WEB_URL}
    prefix: "PROMETHEUS_"
    cache_ttl_seconds: 5  # Cache per 5 secondi
    labels:
      app: web

  - name: worker-app
    type: promphp-redis
    redis_url: ${REDIS_WORKER_URL}
    prefix: "PROMETHEUS_"
    cache_ttl_seconds: 0  # Nessuna cache
    labels:
      app: worker
```

### Parametri del server

| Parametro | Default | Descrizione |
|-----------|---------|-------------|
| `port` | `9090` | Porta del server HTTP |
| `gzip_level` | - | Livello di compressione GZIP 1-9, disabilitato se non impostato |
| `auth.type` | `none` | Tipo di autenticazione: `none`, `basic`, `bearer` |
| `auth.username` | - | Nome utente per basic auth |
| `auth.password` | - | Password per basic auth |
| `auth.token` | - | Token per bearer auth |
| `allowed_ips` | `[]` | Whitelist IP (CIDR), vuoto = consenti tutti |
| `tls.cert` | - | Percorso del certificato TLS |
| `tls.key` | - | Percorso della chiave privata TLS |

### Parametri delle sorgenti

| Parametro | Default | Descrizione |
|-----------|---------|-------------|
| `name` | *richiesto* | Identificatore della sorgente |
| `type` | *richiesto* | Tipo di sorgente: `promphp-redis` |
| `redis_url` | *richiesto* | URL di connessione Redis |
| `prefix` | `PROMETHEUS_` | Prefisso delle chiavi Redis |
| `cache_ttl_seconds` | `0` | TTL della cache in secondi (0 = disabilitato) |
| `label_format` | `auto` | Formato delle etichette: `auto`, `json`, `base64` |
| `labels` | `{}` | Etichette aggiuntive per tutte le metriche |

## Endpoint

| Endpoint | Autenticazione | Descrizione |
|----------|----------------|-------------|
| `GET /metrics` | Sì | Metriche Prometheus |
| `GET /health` | No | Controllo salute (sempre 200) |
| `GET /ready` | No | Controllo prontezza (200 se tutte le sorgenti sono sane) |

## Metriche proprie

L'esportatore espone metriche su se stesso:

```
# Durata scrape per sorgente
metrics_bridge_scrape_duration_seconds{source="web-app"} 0.003

# Errori scrape per sorgente
metrics_bridge_scrape_errors_total{source="web-app"} 0

# Salute della sorgente (1=attiva, 0=inattiva)
metrics_bridge_source_up{source="web-app"} 1

# Totale sorgenti configurate
metrics_bridge_sources_total 2

# Stato dell'esportatore
metrics_bridge_up 1

# Informazioni build
metrics_bridge_build_info{version="0.1.0",rust_version="1.84"} 1
```

## Compilazione

```bash
# Compilare
cargo build --release

# Test
cargo test

# Linter
cargo clippy --all-targets -- -D warnings

# Immagine Docker
docker build -t metrics-bridge .
```

## Licenza

MIT License - vedi [LICENSE](LICENSE) per i dettagli.

## Contributi

Benvenuti! Per favore leggi prima [CONTRIBUTING.md](CONTRIBUTING.md).
