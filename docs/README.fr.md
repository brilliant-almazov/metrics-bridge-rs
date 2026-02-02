# metrics-bridge

[English](../README.md) | [Русский](README.ru.md) | [中文](README.zh.md) | [हिंदी](README.hi.md) | [Español](README.es.md) | [Português](README.pt.md) | **Français** | [Deutsch](README.de.md) | [日本語](README.ja.md) | [한국어](README.ko.md)

[![Test](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/test.yml/badge.svg)](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/test.yml)
[![Release](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/release.yml/badge.svg)](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/release.yml)
[![codecov](https://codecov.io/gh/brilliant-almazov/metrics-bridge-rs/graph/badge.svg)](https://codecov.io/gh/brilliant-almazov/metrics-bridge-rs)
[![Docker Image Size](https://ghcr-badge.egpl.dev/brilliant-almazov/metrics-bridge-rs/size)](https://github.com/brilliant-almazov/metrics-bridge-rs/pkgs/container/metrics-bridge-rs)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Exportateur de métriques Prometheus haute performance** pour le stockage Redis de [promphp/prometheus_client_php](https://github.com/promphp/prometheus_client_php). Écrit en Rust pour des performances maximales et une consommation minimale de ressources.

## Fonctionnalités principales

- **Ultra rapide** - Temps de réponse sub-milliseconde (vs 50-200ms via PHP)
- **Haut débit** - 12 500+ requêtes/seconde avec cache activé
- **Sources multiples** - Agrégation de métriques depuis plusieurs instances Redis
- **Cache flexible** - TTL de cache individuel par source
- **Labels personnalisés** - Ajoutez des labels supplémentaires aux métriques (app, environment, etc.)
- **Sécurité** - Basic Auth, Bearer Token, liste blanche IP, TLS
- **Compression** - Compression GZIP optionnelle
- **Auto-surveillance** - Métriques intégrées de l'exportateur lui-même
- **Image compacte** - Image Docker <10Mo (basée sur scratch)
- **Compatibilité** - Fonctionne avec Redis, Dragonfly, Valkey, KeyDB

## Problème

Les applications PHP utilisant promphp/prometheus_client_php écrivent des métriques dans Redis. La lecture de ces métriques via PHP est lente en raison de la surcharge du framework (~50-200ms avec Symfony).

**metrics-bridge** lit les métriques directement depuis Redis et les sert à Prometheus avec une latence sub-milliseconde.

## Performances

### Avec cache (recommandé)

| Métrique | Valeur |
|----------|--------|
| Latence moyenne | **0.8ms** |
| Latence P99 | 7.7ms |
| Débit | **12 500+ req/s** |

### Sans cache

| Métrique | Valeur |
|----------|--------|
| Latence moyenne | ~19ms |
| Débit | ~50 req/s |

## Démarrage rapide

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

Créez le fichier `config.yaml` :

```yaml
server:
  port: 9090

  # Optionnel : compression GZIP
  gzip_level: 6  # 1-9 (1=plus rapide, 9=meilleure compression)

  auth:
    type: basic  # none | basic | bearer
    username: prometheus
    password: ${METRICS_PASSWORD}  # Substitution de variables d'environnement

  # Optionnel : liste blanche IP
  allowed_ips:
    - 10.0.0.0/8
    - 192.168.0.0/16

  # Optionnel : TLS
  tls:
    cert: /path/to/cert.pem
    key: /path/to/key.pem

sources:
  - name: web-app
    type: promphp-redis
    redis_url: ${REDIS_WEB_URL}
    prefix: "PROMETHEUS_"
    cache_ttl_seconds: 5  # Cache pendant 5 secondes
    labels:
      app: web

  - name: worker-app
    type: promphp-redis
    redis_url: ${REDIS_WORKER_URL}
    prefix: "PROMETHEUS_"
    cache_ttl_seconds: 0  # Pas de cache
    labels:
      app: worker
```

### Paramètres du serveur

| Paramètre | Par défaut | Description |
|-----------|------------|-------------|
| `port` | `9090` | Port du serveur HTTP |
| `gzip_level` | - | Niveau de compression GZIP 1-9, désactivé si non défini |
| `auth.type` | `none` | Type d'authentification : `none`, `basic`, `bearer` |
| `auth.username` | - | Nom d'utilisateur pour basic auth |
| `auth.password` | - | Mot de passe pour basic auth |
| `auth.token` | - | Token pour bearer auth |
| `allowed_ips` | `[]` | Liste blanche IP (CIDR), vide = tout autoriser |
| `tls.cert` | - | Chemin vers le certificat TLS |
| `tls.key` | - | Chemin vers la clé privée TLS |

### Paramètres des sources

| Paramètre | Par défaut | Description |
|-----------|------------|-------------|
| `name` | *requis* | Identifiant de la source |
| `type` | *requis* | Type de source : `promphp-redis` |
| `redis_url` | *requis* | URL de connexion Redis |
| `prefix` | `PROMETHEUS_` | Préfixe des clés Redis |
| `cache_ttl_seconds` | `0` | TTL du cache en secondes (0 = désactivé) |
| `label_format` | `auto` | Format des labels : `auto`, `json`, `base64` |
| `labels` | `{}` | Labels supplémentaires pour toutes les métriques |

## Points de terminaison

| Endpoint | Authentification | Description |
|----------|------------------|-------------|
| `GET /metrics` | Oui | Métriques Prometheus |
| `GET /health` | Non | Vérification de santé (toujours 200) |
| `GET /ready` | Non | Vérification de disponibilité (200 si toutes les sources sont saines) |

## Métriques propres

L'exportateur expose des métriques sur lui-même :

```
# Durée de scrape par source
metrics_bridge_scrape_duration_seconds{source="web-app"} 0.003

# Erreurs de scrape par source
metrics_bridge_scrape_errors_total{source="web-app"} 0

# Santé de la source (1=active, 0=inactive)
metrics_bridge_source_up{source="web-app"} 1

# Nombre total de sources configurées
metrics_bridge_sources_total 2

# Statut de l'exportateur
metrics_bridge_up 1

# Informations de build
metrics_bridge_build_info{version="0.1.0",rust_version="1.84"} 1
```

## Compilation

```bash
# Compiler
cargo build --release

# Tests
cargo test

# Linter
cargo clippy --all-targets -- -D warnings

# Image Docker
docker build -t metrics-bridge .
```

## Licence

MIT License - voir [LICENSE](LICENSE) pour les détails.

## Contributions

Bienvenues ! Veuillez d'abord lire [CONTRIBUTING.md](CONTRIBUTING.md).
