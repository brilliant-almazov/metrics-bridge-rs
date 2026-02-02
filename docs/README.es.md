# metrics-bridge

[English](../README.md) | [Русский](README.ru.md) | [中文](README.zh.md) | [हिंदी](README.hi.md) | **Español** | [Português](README.pt.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Polski](README.pl.md) | [Nederlands](README.nl.md) | [Italiano](README.it.md) | [Türkçe](README.tr.md) | [Українська](README.uk.md) | [Bahasa Indonesia](README.id.md) | [Tiếng Việt](README.vi.md) | [Svenska](README.sv.md) | [Suomi](README.fi.md)

[![Test](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/test.yml/badge.svg)](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/test.yml)
[![Release](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/release.yml/badge.svg)](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/release.yml)
[![codecov](https://codecov.io/gh/brilliant-almazov/metrics-bridge-rs/graph/badge.svg)](https://codecov.io/gh/brilliant-almazov/metrics-bridge-rs)
[![Docker Image Size](https://ghcr-badge.egpl.dev/brilliant-almazov/metrics-bridge-rs/size)](https://github.com/brilliant-almazov/metrics-bridge-rs/pkgs/container/metrics-bridge-rs)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Exportador de métricas Prometheus de alto rendimiento** para almacenamiento Redis de [promphp/prometheus_client_php](https://github.com/promphp/prometheus_client_php). Escrito en Rust para máximo rendimiento y mínimo consumo de recursos.

## Características principales

- **Ultra rápido** - Tiempo de respuesta sub-milisegundo (vs 50-200ms a través de PHP)
- **Alto throughput** - 12,500+ solicitudes/segundo con caché habilitado
- **Múltiples fuentes** - Agregación de métricas desde múltiples instancias Redis
- **Caché flexible** - TTL de caché individual por fuente
- **Etiquetas personalizadas** - Añade etiquetas adicionales a métricas (app, environment, etc.)
- **Seguridad** - Basic Auth, Bearer Token, lista blanca IP, TLS
- **Compresión** - Compresión GZIP opcional
- **Auto-monitoreo** - Métricas integradas del propio exportador
- **Imagen compacta** - Imagen Docker <10MB (basada en scratch)
- **Compatibilidad** - Funciona con Redis, Dragonfly, Valkey, KeyDB

## Problema

Las aplicaciones PHP que usan promphp/prometheus_client_php escriben métricas en Redis. Leer estas métricas a través de PHP es lento debido a la sobrecarga del framework (~50-200ms en Symfony).

**metrics-bridge** lee las métricas directamente desde Redis y las sirve a Prometheus con latencia sub-milisegundo.

## Rendimiento

### Con caché (recomendado)

| Métrica | Valor |
|---------|-------|
| Latencia media | **0.8ms** |
| Latencia P99 | 7.7ms |
| Throughput | **12,500+ req/s** |

### Sin caché

| Métrica | Valor |
|---------|-------|
| Latencia media | ~19ms |
| Throughput | ~50 req/s |

## Inicio rápido

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

## Configuración

Crea el archivo `config.yaml`:

```yaml
server:
  port: 9090

  # Opcional: compresión GZIP
  gzip_level: 6  # 1-9 (1=más rápido, 9=mejor compresión)

  auth:
    type: basic  # none | basic | bearer
    username: prometheus
    password: ${METRICS_PASSWORD}  # Sustitución de variables de entorno

  # Opcional: lista blanca IP
  allowed_ips:
    - 10.0.0.0/8
    - 192.168.0.0/16

  # Opcional: TLS
  tls:
    cert: /path/to/cert.pem
    key: /path/to/key.pem

sources:
  - name: web-app
    type: promphp-redis
    redis_url: ${REDIS_WEB_URL}
    prefix: "PROMETHEUS_"
    cache_ttl_seconds: 5  # Caché por 5 segundos
    labels:
      app: web

  - name: worker-app
    type: promphp-redis
    redis_url: ${REDIS_WORKER_URL}
    prefix: "PROMETHEUS_"
    cache_ttl_seconds: 0  # Sin caché
    labels:
      app: worker
```

### Parámetros del servidor

| Parámetro | Por defecto | Descripción |
|-----------|-------------|-------------|
| `port` | `9090` | Puerto del servidor HTTP |
| `gzip_level` | - | Nivel de compresión GZIP 1-9, deshabilitado si no se establece |
| `auth.type` | `none` | Tipo de autenticación: `none`, `basic`, `bearer` |
| `auth.username` | - | Usuario para basic auth |
| `auth.password` | - | Contraseña para basic auth |
| `auth.token` | - | Token para bearer auth |
| `allowed_ips` | `[]` | Lista blanca IP (CIDR), vacío = permitir todo |
| `tls.cert` | - | Ruta al certificado TLS |
| `tls.key` | - | Ruta a la clave privada TLS |

### Parámetros de fuentes

| Parámetro | Por defecto | Descripción |
|-----------|-------------|-------------|
| `name` | *requerido* | Identificador de la fuente |
| `type` | *requerido* | Tipo de fuente: `promphp-redis` |
| `redis_url` | *requerido* | URL de conexión Redis |
| `prefix` | `PROMETHEUS_` | Prefijo de claves Redis |
| `cache_ttl_seconds` | `0` | TTL de caché en segundos (0 = deshabilitado) |
| `label_format` | `auto` | Formato de etiquetas: `auto`, `json`, `base64` |
| `labels` | `{}` | Etiquetas adicionales para todas las métricas |

## Endpoints

| Endpoint | Autenticación | Descripción |
|----------|---------------|-------------|
| `GET /metrics` | Sí | Métricas Prometheus |
| `GET /health` | No | Verificación de salud (siempre 200) |
| `GET /ready` | No | Verificación de disponibilidad (200 si todas las fuentes están sanas) |

## Métricas propias

El exportador expone métricas sobre sí mismo:

```
# Duración de scrape por fuente
metrics_bridge_scrape_duration_seconds{source="web-app"} 0.003

# Errores de scrape por fuente
metrics_bridge_scrape_errors_total{source="web-app"} 0

# Salud de la fuente (1=activa, 0=caída)
metrics_bridge_source_up{source="web-app"} 1

# Total de fuentes configuradas
metrics_bridge_sources_total 2

# Estado del exportador
metrics_bridge_up 1

# Información de compilación
metrics_bridge_build_info{version="0.1.0",rust_version="1.84"} 1
```

## Compilación

```bash
# Compilar
cargo build --release

# Tests
cargo test

# Linter
cargo clippy --all-targets -- -D warnings

# Imagen Docker
docker build -t metrics-bridge .
```

## Licencia

MIT License - ver [LICENSE](LICENSE) para detalles.

## Contribuciones

¡Bienvenidas! Por favor lee [CONTRIBUTING.md](CONTRIBUTING.md) primero.
