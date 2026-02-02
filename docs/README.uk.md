# metrics-bridge

[English](../README.md) | [Русский](README.ru.md) | [中文](README.zh.md) | [Español](README.es.md) | [Português](README.pt.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Polski](README.pl.md) | [Nederlands](README.nl.md) | [Italiano](README.it.md) | [Türkçe](README.tr.md) | **Українська**

[![Test](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/test.yml/badge.svg)](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/test.yml)
[![Release](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/release.yml/badge.svg)](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/release.yml)
[![codecov](https://codecov.io/gh/brilliant-almazov/metrics-bridge-rs/graph/badge.svg)](https://codecov.io/gh/brilliant-almazov/metrics-bridge-rs)
[![Docker Image Size](https://ghcr-badge.egpl.dev/brilliant-almazov/metrics-bridge-rs/size)](https://github.com/brilliant-almazov/metrics-bridge-rs/pkgs/container/metrics-bridge-rs)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Високопродуктивний експортер метрик Prometheus** для [promphp/prometheus_client_php](https://github.com/promphp/prometheus_client_php) Redis-сховища. Написаний на Rust для максимальної продуктивності та мінімального споживання ресурсів.

## Основні можливості

- **Надшвидкий** - Час відгуку менше мілісекунди (проти 50-200мс через PHP)
- **Висока пропускна здатність** - 12 500+ запитів на секунду з увімкненим кешуванням
- **Множинні джерела** - Агрегація метрик з декількох Redis-інстансів
- **Гнучке кешування** - Індивідуальний TTL кешу для кожного джерела
- **Додаткові мітки** - Додавання довільних міток до метрик (app, environment тощо)
- **Безпека** - Basic Auth, Bearer Token, IP-whitelist, TLS
- **Стиснення** - Опціональне GZIP-стиснення
- **Самомоніторинг** - Вбудовані метрики самого експортера
- **Компактний образ** - Docker-образ менше 10MB (на основі scratch)
- **Сумісність** - Працює з Redis, Dragonfly, Valkey, KeyDB

## Проблема

PHP-застосунки, що використовують promphp/prometheus_client_php, записують метрики в Redis. Читання цих метрик через PHP відбувається повільно через накладні витрати фреймворку (~50-200мс в Symfony).

**metrics-bridge** читає метрики напряму з Redis та віддає їх Prometheus із затримкою менше мілісекунди.

## Продуктивність

### З кешуванням (рекомендовано)

| Метрика | Значення |
|---------|----------|
| Середня затримка | **0.8мс** |
| P99 затримка | 7.7мс |
| Пропускна здатність | **12 500+ req/s** |

### Без кешування

| Метрика | Значення |
|---------|----------|
| Середня затримка | ~19мс |
| Пропускна здатність | ~50 req/s |

## Швидкий старт

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

## Конфігурація

Створіть файл `config.yaml`:

```yaml
server:
  port: 9090

  # Опціонально: GZIP-стиснення
  gzip_level: 6  # 1-9 (1=швидше, 9=краще стиснення)

  auth:
    type: basic  # none | basic | bearer
    username: prometheus
    password: ${METRICS_PASSWORD}  # Підстановка змінних оточення

  # Опціонально: IP whitelist
  allowed_ips:
    - 10.0.0.0/8
    - 192.168.0.0/16

  # Опціонально: TLS
  tls:
    cert: /path/to/cert.pem
    key: /path/to/key.pem

sources:
  - name: web-app
    type: promphp-redis
    redis_url: ${REDIS_WEB_URL}
    prefix: "PROMETHEUS_"
    cache_ttl_seconds: 5  # Кешування на 5 секунд
    labels:
      app: web

  - name: worker-app
    type: promphp-redis
    redis_url: ${REDIS_WORKER_URL}
    prefix: "PROMETHEUS_"
    cache_ttl_seconds: 0  # Без кешування
    labels:
      app: worker
```

### Параметри сервера

| Параметр | За замовчуванням | Опис |
|----------|------------------|------|
| `port` | `9090` | Порт HTTP-сервера |
| `gzip_level` | - | Рівень GZIP-стиснення 1-9, вимкнено якщо не вказано |
| `auth.type` | `none` | Тип автентифікації: `none`, `basic`, `bearer` |
| `auth.username` | - | Логін для basic auth |
| `auth.password` | - | Пароль для basic auth |
| `auth.token` | - | Токен для bearer auth |
| `allowed_ips` | `[]` | IP whitelist (CIDR), порожній = дозволити все |
| `tls.cert` | - | Шлях до TLS-сертифіката |
| `tls.key` | - | Шлях до приватного ключа TLS |

### Параметри джерел

| Параметр | За замовчуванням | Опис |
|----------|------------------|------|
| `name` | *обов'язково* | Ідентифікатор джерела |
| `type` | *обов'язково* | Тип джерела: `promphp-redis` |
| `redis_url` | *обов'язково* | URL підключення до Redis |
| `prefix` | `PROMETHEUS_` | Префікс ключів в Redis |
| `cache_ttl_seconds` | `0` | TTL кешу в секундах (0 = вимкнено) |
| `label_format` | `auto` | Формат міток: `auto`, `json`, `base64` |
| `labels` | `{}` | Додаткові мітки для всіх метрик |

## Ендпоінти

| Ендпоінт | Автентифікація | Опис |
|----------|----------------|------|
| `GET /metrics` | Так | Метрики Prometheus |
| `GET /health` | Ні | Перевірка працездатності (завжди 200) |
| `GET /ready` | Ні | Готовність (200 якщо всі джерела здорові) |

## Власні метрики

Експортер надає метрики про себе:

```
# Тривалість збору метрик по джерелу
metrics_bridge_scrape_duration_seconds{source="web-app"} 0.003

# Помилки збору по джерелу
metrics_bridge_scrape_errors_total{source="web-app"} 0

# Здоров'я джерела (1=працює, 0=недоступне)
metrics_bridge_source_up{source="web-app"} 1

# Всього налаштованих джерел
metrics_bridge_sources_total 2

# Статус експортера
metrics_bridge_up 1

# Інформація про збірку
metrics_bridge_build_info{version="0.1.0",rust_version="1.84"} 1
```

## Збірка

```bash
# Збірка
cargo build --release

# Тести
cargo test

# Лінтер
cargo clippy --all-targets -- -D warnings

# Docker образ
docker build -t metrics-bridge .
```

## Ліцензія

MIT License - див. файл [LICENSE](LICENSE).

## Участь у розробці

Вітаємо! Будь ласка, спочатку прочитайте [CONTRIBUTING.md](CONTRIBUTING.md).
