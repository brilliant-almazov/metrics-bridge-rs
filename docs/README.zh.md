# metrics-bridge

[English](../README.md) | [Русский](README.ru.md) | **中文** | [हिंदी](README.hi.md) | [Español](README.es.md) | [Português](README.pt.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [日本語](README.ja.md) | [한국어](README.ko.md)

[![Test](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/test.yml/badge.svg)](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/test.yml)
[![Release](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/release.yml/badge.svg)](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/release.yml)
[![codecov](https://codecov.io/gh/brilliant-almazov/metrics-bridge-rs/graph/badge.svg)](https://codecov.io/gh/brilliant-almazov/metrics-bridge-rs)
[![Docker Image Size](https://ghcr-badge.egpl.dev/brilliant-almazov/metrics-bridge-rs/size)](https://github.com/brilliant-almazov/metrics-bridge-rs/pkgs/container/metrics-bridge-rs)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**高性能 Prometheus 指标导出器**，用于 [promphp/prometheus_client_php](https://github.com/promphp/prometheus_client_php) Redis 存储。使用 Rust 编写，追求极致性能和最小资源消耗。

## 主要特性

- **极速响应** - 亚毫秒级响应时间（相比 PHP 的 50-200ms）
- **高吞吐量** - 启用缓存时支持 12,500+ 请求/秒
- **多数据源** - 从多个 Redis 实例聚合指标
- **灵活缓存** - 每个数据源独立的缓存 TTL
- **自定义标签** - 为指标添加额外标签（app、environment 等）
- **安全认证** - Basic Auth、Bearer Token、IP 白名单、TLS
- **响应压缩** - 可选的 GZIP 压缩
- **自我监控** - 内置导出器自身的监控指标
- **精简镜像** - Docker 镜像小于 10MB（基于 scratch）
- **广泛兼容** - 支持 Redis、Dragonfly、Valkey、KeyDB

## 问题背景

使用 promphp/prometheus_client_php 的 PHP 应用将指标写入 Redis。通过 PHP 读取这些指标很慢，因为框架开销大（Symfony 约 50-200ms）。

**metrics-bridge** 直接从 Redis 读取指标，以亚毫秒级延迟提供给 Prometheus。

## 性能数据

### 启用缓存（推荐）

| 指标 | 数值 |
|------|------|
| 平均延迟 | **0.8ms** |
| P99 延迟 | 7.7ms |
| 吞吐量 | **12,500+ req/s** |

### 无缓存

| 指标 | 数值 |
|------|------|
| 平均延迟 | ~19ms |
| 吞吐量 | ~50 req/s |

## 快速开始

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

## 配置

创建 `config.yaml` 文件：

```yaml
server:
  port: 9090

  # 可选：GZIP 压缩
  gzip_level: 6  # 1-9（1=最快，9=最佳压缩）

  auth:
    type: basic  # none | basic | bearer
    username: prometheus
    password: ${METRICS_PASSWORD}  # 支持环境变量替换

  # 可选：IP 白名单
  allowed_ips:
    - 10.0.0.0/8
    - 192.168.0.0/16

  # 可选：TLS
  tls:
    cert: /path/to/cert.pem
    key: /path/to/key.pem

sources:
  - name: web-app
    type: promphp-redis
    redis_url: ${REDIS_WEB_URL}
    prefix: "PROMETHEUS_"
    cache_ttl_seconds: 5  # 缓存 5 秒
    labels:
      app: web

  - name: worker-app
    type: promphp-redis
    redis_url: ${REDIS_WORKER_URL}
    prefix: "PROMETHEUS_"
    cache_ttl_seconds: 0  # 不缓存
    labels:
      app: worker
```

### 服务器参数

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `port` | `9090` | HTTP 服务器端口 |
| `gzip_level` | - | GZIP 压缩级别 1-9，不设置则禁用 |
| `auth.type` | `none` | 认证类型：`none`、`basic`、`bearer` |
| `auth.username` | - | Basic 认证用户名 |
| `auth.password` | - | Basic 认证密码 |
| `auth.token` | - | Bearer 认证令牌 |
| `allowed_ips` | `[]` | IP 白名单（CIDR），空=允许全部 |
| `tls.cert` | - | TLS 证书路径 |
| `tls.key` | - | TLS 私钥路径 |

### 数据源参数

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `name` | *必需* | 数据源标识符 |
| `type` | *必需* | 数据源类型：`promphp-redis` |
| `redis_url` | *必需* | Redis 连接 URL |
| `prefix` | `PROMETHEUS_` | Redis 键前缀 |
| `cache_ttl_seconds` | `0` | 缓存 TTL 秒数（0=禁用） |
| `label_format` | `auto` | 标签格式：`auto`、`json`、`base64` |
| `labels` | `{}` | 添加到所有指标的额外标签 |

## 接口端点

| 端点 | 认证 | 说明 |
|------|------|------|
| `GET /metrics` | 是 | Prometheus 指标 |
| `GET /health` | 否 | 健康检查（始终返回 200） |
| `GET /ready` | 否 | 就绪检查（所有数据源健康时返回 200） |

## 自身指标

导出器提供自身的监控指标：

```
# 每个数据源的抓取耗时
metrics_bridge_scrape_duration_seconds{source="web-app"} 0.003

# 每个数据源的抓取错误数
metrics_bridge_scrape_errors_total{source="web-app"} 0

# 数据源健康状态（1=正常，0=异常）
metrics_bridge_source_up{source="web-app"} 1

# 配置的数据源总数
metrics_bridge_sources_total 2

# 导出器状态
metrics_bridge_up 1

# 构建信息
metrics_bridge_build_info{version="0.1.0",rust_version="1.84"} 1
```

## 构建

```bash
# 构建
cargo build --release

# 测试
cargo test

# 代码检查
cargo clippy --all-targets -- -D warnings

# Docker 镜像
docker build -t metrics-bridge .
```

## 许可证

MIT License - 详见 [LICENSE](LICENSE) 文件。

## 参与贡献

欢迎贡献！请先阅读 [CONTRIBUTING.md](CONTRIBUTING.md)。
