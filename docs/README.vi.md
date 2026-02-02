# metrics-bridge

[English](../README.md) | [Русский](README.ru.md) | [中文](README.zh.md) | [Español](README.es.md) | [Português](README.pt.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Polski](README.pl.md) | [Nederlands](README.nl.md) | [Italiano](README.it.md) | [Türkçe](README.tr.md) | [Українська](README.uk.md) | [Bahasa Indonesia](README.id.md) | **Tiếng Việt**

[![Test](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/test.yml/badge.svg)](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/test.yml)
[![Release](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/release.yml/badge.svg)](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/release.yml)
[![codecov](https://codecov.io/gh/brilliant-almazov/metrics-bridge-rs/graph/badge.svg)](https://codecov.io/gh/brilliant-almazov/metrics-bridge-rs)
[![Docker Image Size](https://ghcr-badge.egpl.dev/brilliant-almazov/metrics-bridge-rs/size)](https://github.com/brilliant-almazov/metrics-bridge-rs/pkgs/container/metrics-bridge-rs)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Trình xuất metrics Prometheus hiệu suất cao** cho lưu trữ Redis [promphp/prometheus_client_php](https://github.com/promphp/prometheus_client_php). Viết bằng Rust để đạt hiệu suất tối đa và tiêu thụ tài nguyên tối thiểu.

## Tính năng chính

- **Siêu nhanh** - Thời gian phản hồi dưới mili giây (so với 50-200ms qua PHP)
- **Thông lượng cao** - 12.500+ yêu cầu/giây với cache được bật
- **Đa nguồn** - Tổng hợp metrics từ nhiều instance Redis
- **Cache linh hoạt** - TTL cache riêng biệt cho từng nguồn
- **Label tùy chỉnh** - Thêm label bổ sung vào metrics (app, environment, v.v.)
- **Bảo mật** - Basic Auth, Bearer Token, whitelist IP, TLS
- **Nén** - Nén GZIP tùy chọn
- **Tự giám sát** - Metrics tích hợp của chính exporter
- **Image nhỏ gọn** - Image Docker <10MB (dựa trên scratch)
- **Tương thích** - Hoạt động với Redis, Dragonfly, Valkey, KeyDB

## Vấn đề

Các ứng dụng PHP sử dụng promphp/prometheus_client_php ghi metrics vào Redis. Đọc các metrics này qua PHP chậm do overhead của framework (~50-200ms với Symfony).

**metrics-bridge** đọc metrics trực tiếp từ Redis và phục vụ cho Prometheus với độ trễ dưới mili giây.

## Hiệu suất

### Với cache (khuyến nghị)

| Metric | Giá trị |
|--------|---------|
| Độ trễ trung bình | **0.8ms** |
| Độ trễ P99 | 7.7ms |
| Thông lượng | **12.500+ req/s** |

### Không cache

| Metric | Giá trị |
|--------|---------|
| Độ trễ trung bình | ~19ms |
| Thông lượng | ~50 req/s |

## Bắt đầu nhanh

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

## Cấu hình

Tạo file `config.yaml`:

```yaml
server:
  port: 9090

  # Tùy chọn: nén GZIP
  gzip_level: 6  # 1-9 (1=nhanh nhất, 9=nén tốt nhất)

  auth:
    type: basic  # none | basic | bearer
    username: prometheus
    password: ${METRICS_PASSWORD}  # Thay thế biến môi trường

  # Tùy chọn: whitelist IP
  allowed_ips:
    - 10.0.0.0/8
    - 192.168.0.0/16

  # Tùy chọn: TLS
  tls:
    cert: /path/to/cert.pem
    key: /path/to/key.pem

sources:
  - name: web-app
    type: promphp-redis
    redis_url: ${REDIS_WEB_URL}
    prefix: "PROMETHEUS_"
    cache_ttl_seconds: 5  # Cache trong 5 giây
    labels:
      app: web

  - name: worker-app
    type: promphp-redis
    redis_url: ${REDIS_WORKER_URL}
    prefix: "PROMETHEUS_"
    cache_ttl_seconds: 0  # Không cache
    labels:
      app: worker
```

### Tham số server

| Tham số | Mặc định | Mô tả |
|---------|----------|-------|
| `port` | `9090` | Cổng server HTTP |
| `gzip_level` | - | Mức nén GZIP 1-9, tắt nếu không đặt |
| `auth.type` | `none` | Loại xác thực: `none`, `basic`, `bearer` |
| `auth.username` | - | Username cho basic auth |
| `auth.password` | - | Password cho basic auth |
| `auth.token` | - | Token cho bearer auth |
| `allowed_ips` | `[]` | Whitelist IP (CIDR), rỗng = cho phép tất cả |
| `tls.cert` | - | Đường dẫn đến chứng chỉ TLS |
| `tls.key` | - | Đường dẫn đến khóa riêng TLS |

### Tham số nguồn

| Tham số | Mặc định | Mô tả |
|---------|----------|-------|
| `name` | *bắt buộc* | Định danh nguồn |
| `type` | *bắt buộc* | Loại nguồn: `promphp-redis` |
| `redis_url` | *bắt buộc* | URL kết nối Redis |
| `prefix` | `PROMETHEUS_` | Tiền tố khóa Redis |
| `cache_ttl_seconds` | `0` | TTL cache tính bằng giây (0 = tắt) |
| `label_format` | `auto` | Định dạng label: `auto`, `json`, `base64` |
| `labels` | `{}` | Label bổ sung cho tất cả metrics |

## Endpoint

| Endpoint | Xác thực | Mô tả |
|----------|----------|-------|
| `GET /metrics` | Có | Metrics Prometheus |
| `GET /health` | Không | Kiểm tra sức khỏe (luôn 200) |
| `GET /ready` | Không | Kiểm tra sẵn sàng (200 nếu tất cả nguồn khỏe mạnh) |

## Metrics riêng

Exporter cung cấp metrics về chính nó:

```
# Thời gian scrape mỗi nguồn
metrics_bridge_scrape_duration_seconds{source="web-app"} 0.003

# Lỗi scrape mỗi nguồn
metrics_bridge_scrape_errors_total{source="web-app"} 0

# Sức khỏe nguồn (1=hoạt động, 0=không hoạt động)
metrics_bridge_source_up{source="web-app"} 1

# Tổng số nguồn được cấu hình
metrics_bridge_sources_total 2

# Trạng thái exporter
metrics_bridge_up 1

# Thông tin build
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

## Giấy phép

MIT License - xem [LICENSE](LICENSE) để biết chi tiết.

## Đóng góp

Chào mừng! Vui lòng đọc [CONTRIBUTING.md](CONTRIBUTING.md) trước.
