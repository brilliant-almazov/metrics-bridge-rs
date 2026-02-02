# metrics-bridge

[English](../README.md) | [Русский](README.ru.md) | [中文](README.zh.md) | [Español](README.es.md) | [Português](README.pt.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Polski](README.pl.md) | [Nederlands](README.nl.md) | [Italiano](README.it.md) | [Türkçe](README.tr.md) | [Українська](README.uk.md) | **Bahasa Indonesia** | [Tiếng Việt](README.vi.md)

[![Test](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/test.yml/badge.svg)](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/test.yml)
[![Release](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/release.yml/badge.svg)](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/release.yml)
[![codecov](https://codecov.io/gh/brilliant-almazov/metrics-bridge-rs/graph/badge.svg)](https://codecov.io/gh/brilliant-almazov/metrics-bridge-rs)
[![Docker Image Size](https://ghcr-badge.egpl.dev/brilliant-almazov/metrics-bridge-rs/size)](https://github.com/brilliant-almazov/metrics-bridge-rs/pkgs/container/metrics-bridge-rs)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Eksporter metrik Prometheus berkinerja tinggi** untuk penyimpanan Redis [promphp/prometheus_client_php](https://github.com/promphp/prometheus_client_php). Ditulis dalam Rust untuk kinerja maksimal dan konsumsi sumber daya minimal.

## Fitur utama

- **Sangat cepat** - Waktu respons sub-milidetik (vs 50-200ms melalui PHP)
- **Throughput tinggi** - 12.500+ permintaan/detik dengan cache diaktifkan
- **Multi sumber** - Agregasi metrik dari beberapa instance Redis
- **Cache fleksibel** - TTL cache individual per sumber
- **Label kustom** - Tambahkan label tambahan ke metrik (app, environment, dll.)
- **Keamanan** - Basic Auth, Bearer Token, whitelist IP, TLS
- **Kompresi** - Kompresi GZIP opsional
- **Pemantauan mandiri** - Metrik bawaan dari eksporter itu sendiri
- **Image kompak** - Image Docker <10MB (berbasis scratch)
- **Kompatibilitas** - Bekerja dengan Redis, Dragonfly, Valkey, KeyDB

## Masalah

Aplikasi PHP yang menggunakan promphp/prometheus_client_php menulis metrik ke Redis. Membaca metrik ini melalui PHP lambat karena overhead framework (~50-200ms dengan Symfony).

**metrics-bridge** membaca metrik langsung dari Redis dan menyajikannya ke Prometheus dengan latensi sub-milidetik.

## Kinerja

### Dengan cache (direkomendasikan)

| Metrik | Nilai |
|--------|-------|
| Latensi rata-rata | **0.8ms** |
| Latensi P99 | 7.7ms |
| Throughput | **12.500+ req/s** |

### Tanpa cache

| Metrik | Nilai |
|--------|-------|
| Latensi rata-rata | ~19ms |
| Throughput | ~50 req/s |

## Mulai cepat

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

## Konfigurasi

Buat file `config.yaml`:

```yaml
server:
  port: 9090

  # Opsional: kompresi GZIP
  gzip_level: 6  # 1-9 (1=tercepat, 9=kompresi terbaik)

  auth:
    type: basic  # none | basic | bearer
    username: prometheus
    password: ${METRICS_PASSWORD}  # Substitusi variabel lingkungan

  # Opsional: whitelist IP
  allowed_ips:
    - 10.0.0.0/8
    - 192.168.0.0/16

  # Opsional: TLS
  tls:
    cert: /path/to/cert.pem
    key: /path/to/key.pem

sources:
  - name: web-app
    type: promphp-redis
    redis_url: ${REDIS_WEB_URL}
    prefix: "PROMETHEUS_"
    cache_ttl_seconds: 5  # Cache selama 5 detik
    labels:
      app: web

  - name: worker-app
    type: promphp-redis
    redis_url: ${REDIS_WORKER_URL}
    prefix: "PROMETHEUS_"
    cache_ttl_seconds: 0  # Tanpa cache
    labels:
      app: worker
```

### Parameter server

| Parameter | Default | Deskripsi |
|-----------|---------|-----------|
| `port` | `9090` | Port server HTTP |
| `gzip_level` | - | Level kompresi GZIP 1-9, dinonaktifkan jika tidak diatur |
| `auth.type` | `none` | Tipe autentikasi: `none`, `basic`, `bearer` |
| `auth.username` | - | Username untuk basic auth |
| `auth.password` | - | Password untuk basic auth |
| `auth.token` | - | Token untuk bearer auth |
| `allowed_ips` | `[]` | Whitelist IP (CIDR), kosong = izinkan semua |
| `tls.cert` | - | Path ke sertifikat TLS |
| `tls.key` | - | Path ke kunci privat TLS |

### Parameter sumber

| Parameter | Default | Deskripsi |
|-----------|---------|-----------|
| `name` | *wajib* | Identifier sumber |
| `type` | *wajib* | Tipe sumber: `promphp-redis` |
| `redis_url` | *wajib* | URL koneksi Redis |
| `prefix` | `PROMETHEUS_` | Prefix kunci Redis |
| `cache_ttl_seconds` | `0` | TTL cache dalam detik (0 = dinonaktifkan) |
| `label_format` | `auto` | Format label: `auto`, `json`, `base64` |
| `labels` | `{}` | Label tambahan untuk semua metrik |

## Endpoint

| Endpoint | Autentikasi | Deskripsi |
|----------|-------------|-----------|
| `GET /metrics` | Ya | Metrik Prometheus |
| `GET /health` | Tidak | Pemeriksaan kesehatan (selalu 200) |
| `GET /ready` | Tidak | Pemeriksaan kesiapan (200 jika semua sumber sehat) |

## Metrik sendiri

Eksporter menyediakan metrik tentang dirinya sendiri:

```
# Durasi scrape per sumber
metrics_bridge_scrape_duration_seconds{source="web-app"} 0.003

# Error scrape per sumber
metrics_bridge_scrape_errors_total{source="web-app"} 0

# Kesehatan sumber (1=aktif, 0=tidak aktif)
metrics_bridge_source_up{source="web-app"} 1

# Total sumber yang dikonfigurasi
metrics_bridge_sources_total 2

# Status eksporter
metrics_bridge_up 1

# Info build
metrics_bridge_build_info{version="0.1.0",rust_version="1.84"} 1
```

## Build

```bash
# Build
cargo build --release

# Test
cargo test

# Linter
cargo clippy --all-targets -- -D warnings

# Image Docker
docker build -t metrics-bridge .
```

## Lisensi

MIT License - lihat [LICENSE](LICENSE) untuk detail.

## Kontribusi

Selamat datang! Silakan baca [CONTRIBUTING.md](CONTRIBUTING.md) terlebih dahulu.
