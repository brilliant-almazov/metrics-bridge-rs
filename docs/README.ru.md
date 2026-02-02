# metrics-bridge

[English](../README.md) | **Русский** | [中文](README.zh.md) | [हिंदी](README.hi.md) | [Español](README.es.md) | [Português](README.pt.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [日本語](README.ja.md) | [한국어](README.ko.md)

[![Test](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/test.yml/badge.svg)](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/test.yml)
[![Release](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/release.yml/badge.svg)](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/release.yml)
[![codecov](https://codecov.io/gh/brilliant-almazov/metrics-bridge-rs/graph/badge.svg)](https://codecov.io/gh/brilliant-almazov/metrics-bridge-rs)
[![Docker Image Size](https://ghcr-badge.egpl.dev/brilliant-almazov/metrics-bridge-rs/size)](https://github.com/brilliant-almazov/metrics-bridge-rs/pkgs/container/metrics-bridge-rs)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Высокопроизводительный экспортер метрик Prometheus** для [promphp/prometheus_client_php](https://github.com/promphp/prometheus_client_php) Redis-хранилища. Написан на Rust для максимальной производительности и минимального потребления ресурсов.

## Основные возможности

- **Сверхбыстрый** - Время отклика менее миллисекунды (против 50-200мс через PHP)
- **Высокая пропускная способность** - 12,500+ запросов в секунду с включённым кешированием
- **Множество источников** - Агрегация метрик из нескольких Redis-инстансов
- **Гибкое кеширование** - Индивидуальный TTL кеша для каждого источника
- **Дополнительные метки** - Добавление произвольных меток к метрикам (app, environment и т.д.)
- **Безопасность** - Basic Auth, Bearer Token, IP-whitelist, TLS
- **Сжатие** - Опциональное GZIP-сжатие ответов
- **Мониторинг** - Встроенные метрики самого экспортера
- **Компактный образ** - Docker-образ менее 10MB (на основе scratch)
- **Совместимость** - Работает с Redis, Dragonfly, Valkey, KeyDB

## Проблема

PHP-приложения, использующие promphp/prometheus_client_php, записывают метрики в Redis. Чтение этих метрик через PHP происходит медленно из-за накладных расходов фреймворка (~50-200мс в Symfony).

**metrics-bridge** читает метрики напрямую из Redis и отдаёт их Prometheus с задержкой менее миллисекунды.

## Производительность

### С кешированием (рекомендуется)

| Метрика | Значение |
|---------|----------|
| Средняя задержка | **0.8мс** |
| P99 задержка | 7.7мс |
| Пропускная способность | **12,500+ req/s** |

### Без кеширования

| Метрика | Значение |
|---------|----------|
| Средняя задержка | ~19мс |
| Пропускная способность | ~50 req/s |

## Быстрый старт

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

## Конфигурация

Создайте файл `config.yaml`:

```yaml
server:
  port: 9090

  # Опционально: GZIP-сжатие
  gzip_level: 6  # 1-9 (1=быстрее, 9=лучше сжатие)

  auth:
    type: basic  # none | basic | bearer
    username: prometheus
    password: ${METRICS_PASSWORD}  # Подстановка переменных окружения

  # Опционально: IP whitelist
  allowed_ips:
    - 10.0.0.0/8
    - 192.168.0.0/16

  # Опционально: TLS
  tls:
    cert: /path/to/cert.pem
    key: /path/to/key.pem

sources:
  - name: web-app
    type: promphp-redis
    redis_url: ${REDIS_WEB_URL}
    prefix: "PROMETHEUS_"
    cache_ttl_seconds: 5  # Кеширование на 5 секунд
    labels:
      app: web

  - name: worker-app
    type: promphp-redis
    redis_url: ${REDIS_WORKER_URL}
    prefix: "PROMETHEUS_"
    cache_ttl_seconds: 0  # Без кеширования
    labels:
      app: worker
```

### Параметры сервера

| Параметр | По умолчанию | Описание |
|----------|--------------|----------|
| `port` | `9090` | Порт HTTP-сервера |
| `gzip_level` | - | Уровень GZIP-сжатия 1-9, отключено если не указано |
| `auth.type` | `none` | Тип аутентификации: `none`, `basic`, `bearer` |
| `auth.username` | - | Логин для basic auth |
| `auth.password` | - | Пароль для basic auth |
| `auth.token` | - | Токен для bearer auth |
| `allowed_ips` | `[]` | IP whitelist (CIDR), пустой = разрешить все |
| `tls.cert` | - | Путь к TLS-сертификату |
| `tls.key` | - | Путь к приватному ключу TLS |

### Параметры источников

| Параметр | По умолчанию | Описание |
|----------|--------------|----------|
| `name` | *обязательно* | Идентификатор источника |
| `type` | *обязательно* | Тип источника: `promphp-redis` |
| `redis_url` | *обязательно* | URL подключения к Redis |
| `prefix` | `PROMETHEUS_` | Префикс ключей в Redis |
| `cache_ttl_seconds` | `0` | TTL кеша в секундах (0 = выключено) |
| `label_format` | `auto` | Формат меток: `auto`, `json`, `base64` |
| `labels` | `{}` | Дополнительные метки для всех метрик |

## Эндпоинты

| Эндпоинт | Аутентификация | Описание |
|----------|----------------|----------|
| `GET /metrics` | Да | Метрики Prometheus |
| `GET /health` | Нет | Проверка работоспособности (всегда 200) |
| `GET /ready` | Нет | Готовность (200 если все источники здоровы) |

## Собственные метрики

Экспортер предоставляет метрики о себе:

```
# Длительность сбора метрик по источнику
metrics_bridge_scrape_duration_seconds{source="web-app"} 0.003

# Ошибки сбора по источнику
metrics_bridge_scrape_errors_total{source="web-app"} 0

# Здоровье источника (1=работает, 0=недоступен)
metrics_bridge_source_up{source="web-app"} 1

# Всего настроенных источников
metrics_bridge_sources_total 2

# Статус экспортера
metrics_bridge_up 1

# Информация о сборке
metrics_bridge_build_info{version="0.1.0",rust_version="1.84"} 1
```

## Сборка

```bash
# Сборка
cargo build --release

# Тесты
cargo test

# Линтер
cargo clippy --all-targets -- -D warnings

# Docker образ
docker build -t metrics-bridge .
```

## Лицензия

MIT License - см. файл [LICENSE](LICENSE).

## Участие в разработке

Приветствуются! Пожалуйста, прочитайте [CONTRIBUTING.md](CONTRIBUTING.md).
