# metrics-bridge

[English](../README.md) | [Русский](README.ru.md) | [中文](README.zh.md) | [Español](README.es.md) | [Português](README.pt.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Polski](README.pl.md) | [Nederlands](README.nl.md) | [Italiano](README.it.md) | **Türkçe** | [Українська](README.uk.md)

[![Test](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/test.yml/badge.svg)](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/test.yml)
[![Release](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/release.yml/badge.svg)](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/release.yml)
[![codecov](https://codecov.io/gh/brilliant-almazov/metrics-bridge-rs/graph/badge.svg)](https://codecov.io/gh/brilliant-almazov/metrics-bridge-rs)
[![Docker Image Size](https://ghcr-badge.egpl.dev/brilliant-almazov/metrics-bridge-rs/size)](https://github.com/brilliant-almazov/metrics-bridge-rs/pkgs/container/metrics-bridge-rs)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Yüksek performanslı Prometheus metrik dışa aktarıcı** - [promphp/prometheus_client_php](https://github.com/promphp/prometheus_client_php) Redis depolaması için. Maksimum performans ve minimum kaynak tüketimi için Rust ile yazılmıştır.

## Ana özellikler

- **Ultra hızlı** - Milisaniyenin altında yanıt süresi (PHP'nin 50-200ms'sine karşı)
- **Yüksek verim** - Önbellek etkinken 12.500+ istek/saniye
- **Çoklu kaynak** - Birden fazla Redis örneğinden metrik toplama
- **Esnek önbellek** - Her kaynak için ayrı önbellek TTL
- **Özel etiketler** - Metriklere ek etiket ekleme (app, environment, vb.)
- **Güvenlik** - Basic Auth, Bearer Token, IP beyaz listesi, TLS
- **Sıkıştırma** - İsteğe bağlı GZIP sıkıştırması
- **Kendi kendini izleme** - Dışa aktarıcının kendi yerleşik metrikleri
- **Kompakt imaj** - <10MB Docker imajı (scratch tabanlı)
- **Uyumluluk** - Redis, Dragonfly, Valkey, KeyDB ile çalışır

## Problem

promphp/prometheus_client_php kullanan PHP uygulamaları metrikleri Redis'e yazar. Bu metrikleri PHP üzerinden okumak framework yükü nedeniyle yavaştır (~50-200ms Symfony'de).

**metrics-bridge** metrikleri doğrudan Redis'ten okur ve milisaniyenin altında gecikmeyle Prometheus'a sunar.

## Performans

### Önbellekli (önerilen)

| Metrik | Değer |
|--------|-------|
| Ortalama gecikme | **0.8ms** |
| P99 gecikme | 7.7ms |
| Verim | **12.500+ req/s** |

### Önbelleksiz

| Metrik | Değer |
|--------|-------|
| Ortalama gecikme | ~19ms |
| Verim | ~50 req/s |

## Hızlı başlangıç

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

## Yapılandırma

`config.yaml` dosyası oluşturun:

```yaml
server:
  port: 9090

  # İsteğe bağlı: GZIP sıkıştırması
  gzip_level: 6  # 1-9 (1=en hızlı, 9=en iyi sıkıştırma)

  auth:
    type: basic  # none | basic | bearer
    username: prometheus
    password: ${METRICS_PASSWORD}  # Ortam değişkeni değiştirme

  # İsteğe bağlı: IP beyaz listesi
  allowed_ips:
    - 10.0.0.0/8
    - 192.168.0.0/16

  # İsteğe bağlı: TLS
  tls:
    cert: /path/to/cert.pem
    key: /path/to/key.pem

sources:
  - name: web-app
    type: promphp-redis
    redis_url: ${REDIS_WEB_URL}
    prefix: "PROMETHEUS_"
    cache_ttl_seconds: 5  # 5 saniye önbellekle
    labels:
      app: web

  - name: worker-app
    type: promphp-redis
    redis_url: ${REDIS_WORKER_URL}
    prefix: "PROMETHEUS_"
    cache_ttl_seconds: 0  # Önbellek yok
    labels:
      app: worker
```

### Sunucu parametreleri

| Parametre | Varsayılan | Açıklama |
|-----------|------------|----------|
| `port` | `9090` | HTTP sunucu portu |
| `gzip_level` | - | GZIP sıkıştırma seviyesi 1-9, ayarlanmazsa devre dışı |
| `auth.type` | `none` | Kimlik doğrulama türü: `none`, `basic`, `bearer` |
| `auth.username` | - | Basic auth için kullanıcı adı |
| `auth.password` | - | Basic auth için şifre |
| `auth.token` | - | Bearer auth için token |
| `allowed_ips` | `[]` | IP beyaz listesi (CIDR), boş = tümüne izin ver |
| `tls.cert` | - | TLS sertifika yolu |
| `tls.key` | - | TLS özel anahtar yolu |

### Kaynak parametreleri

| Parametre | Varsayılan | Açıklama |
|-----------|------------|----------|
| `name` | *gerekli* | Kaynak tanımlayıcı |
| `type` | *gerekli* | Kaynak türü: `promphp-redis` |
| `redis_url` | *gerekli* | Redis bağlantı URL'si |
| `prefix` | `PROMETHEUS_` | Redis anahtar öneki |
| `cache_ttl_seconds` | `0` | Önbellek TTL saniye cinsinden (0 = devre dışı) |
| `label_format` | `auto` | Etiket formatı: `auto`, `json`, `base64` |
| `labels` | `{}` | Tüm metriklere eklenecek ek etiketler |

## Endpoint'ler

| Endpoint | Kimlik doğrulama | Açıklama |
|----------|------------------|----------|
| `GET /metrics` | Evet | Prometheus metrikleri |
| `GET /health` | Hayır | Sağlık kontrolü (her zaman 200) |
| `GET /ready` | Hayır | Hazırlık kontrolü (tüm kaynaklar sağlıklıysa 200) |

## Kendi metrikleri

Dışa aktarıcı kendisi hakkında metrikler sunar:

```
# Kaynak başına scrape süresi
metrics_bridge_scrape_duration_seconds{source="web-app"} 0.003

# Kaynak başına scrape hataları
metrics_bridge_scrape_errors_total{source="web-app"} 0

# Kaynak sağlığı (1=aktif, 0=inaktif)
metrics_bridge_source_up{source="web-app"} 1

# Toplam yapılandırılmış kaynak sayısı
metrics_bridge_sources_total 2

# Dışa aktarıcı durumu
metrics_bridge_up 1

# Build bilgisi
metrics_bridge_build_info{version="0.1.0",rust_version="1.84"} 1
```

## Derleme

```bash
# Derleme
cargo build --release

# Testler
cargo test

# Linter
cargo clippy --all-targets -- -D warnings

# Docker imajı
docker build -t metrics-bridge .
```

## Lisans

MIT License - detaylar için [LICENSE](LICENSE) dosyasına bakın.

## Katkıda bulunma

Hoş geldiniz! Lütfen önce [CONTRIBUTING.md](CONTRIBUTING.md) dosyasını okuyun.
