# metrics-bridge

[English](../README.md) | [Русский](README.ru.md) | [中文](README.zh.md) | [हिंदी](README.hi.md) | [Español](README.es.md) | [Português](README.pt.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | **日本語** | [한국어](README.ko.md)

[![Test](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/test.yml/badge.svg)](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/test.yml)
[![Release](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/release.yml/badge.svg)](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/release.yml)
[![codecov](https://codecov.io/gh/brilliant-almazov/metrics-bridge-rs/graph/badge.svg)](https://codecov.io/gh/brilliant-almazov/metrics-bridge-rs)
[![Docker Image Size](https://ghcr-badge.egpl.dev/brilliant-almazov/metrics-bridge-rs/size)](https://github.com/brilliant-almazov/metrics-bridge-rs/pkgs/container/metrics-bridge-rs)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**高性能Prometheusメトリクスエクスポーター** - [promphp/prometheus_client_php](https://github.com/promphp/prometheus_client_php) Redisストレージ用。最大のパフォーマンスと最小のリソース消費のためにRustで書かれています。

## 主な機能

- **超高速** - サブミリ秒のレスポンスタイム（PHPの50-200msと比較）
- **高スループット** - キャッシュ有効時12,500+リクエスト/秒
- **複数ソース** - 複数のRedisインスタンスからメトリクスを集約
- **柔軟なキャッシュ** - ソースごとの個別キャッシュTTL
- **カスタムラベル** - メトリクスに追加ラベルを付与（app、environmentなど）
- **セキュリティ** - Basic Auth、Bearer Token、IPホワイトリスト、TLS
- **圧縮** - オプションのGZIP圧縮
- **セルフモニタリング** - エクスポーター自身の組み込みメトリクス
- **コンパクトイメージ** - 10MB未満のDockerイメージ（scratchベース）
- **互換性** - Redis、Dragonfly、Valkey、KeyDBで動作

## 課題

promphp/prometheus_client_phpを使用するPHPアプリケーションはRedisにメトリクスを書き込みます。PHP経由でこれらのメトリクスを読み取るとフレームワークのオーバーヘッドにより遅くなります（Symfonyで約50-200ms）。

**metrics-bridge**はRedisから直接メトリクスを読み取り、サブミリ秒のレイテンシでPrometheusに提供します。

## パフォーマンス

### キャッシュあり（推奨）

| メトリック | 値 |
|------------|-----|
| 平均レイテンシ | **0.8ms** |
| P99レイテンシ | 7.7ms |
| スループット | **12,500+ req/s** |

### キャッシュなし

| メトリック | 値 |
|------------|-----|
| 平均レイテンシ | ~19ms |
| スループット | ~50 req/s |

## クイックスタート

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

## 設定

`config.yaml`ファイルを作成：

```yaml
server:
  port: 9090

  # オプション：GZIP圧縮
  gzip_level: 6  # 1-9（1=最速、9=最高圧縮）

  auth:
    type: basic  # none | basic | bearer
    username: prometheus
    password: ${METRICS_PASSWORD}  # 環境変数の置換

  # オプション：IPホワイトリスト
  allowed_ips:
    - 10.0.0.0/8
    - 192.168.0.0/16

  # オプション：TLS
  tls:
    cert: /path/to/cert.pem
    key: /path/to/key.pem

sources:
  - name: web-app
    type: promphp-redis
    redis_url: ${REDIS_WEB_URL}
    prefix: "PROMETHEUS_"
    cache_ttl_seconds: 5  # 5秒間キャッシュ
    labels:
      app: web

  - name: worker-app
    type: promphp-redis
    redis_url: ${REDIS_WORKER_URL}
    prefix: "PROMETHEUS_"
    cache_ttl_seconds: 0  # キャッシュなし
    labels:
      app: worker
```

### サーバーパラメータ

| パラメータ | デフォルト | 説明 |
|------------|-----------|------|
| `port` | `9090` | HTTPサーバーポート |
| `gzip_level` | - | GZIP圧縮レベル1-9、未設定時は無効 |
| `auth.type` | `none` | 認証タイプ：`none`、`basic`、`bearer` |
| `auth.username` | - | Basic認証のユーザー名 |
| `auth.password` | - | Basic認証のパスワード |
| `auth.token` | - | Bearer認証のトークン |
| `allowed_ips` | `[]` | IPホワイトリスト（CIDR）、空=すべて許可 |
| `tls.cert` | - | TLS証明書のパス |
| `tls.key` | - | TLS秘密鍵のパス |

### ソースパラメータ

| パラメータ | デフォルト | 説明 |
|------------|-----------|------|
| `name` | *必須* | ソース識別子 |
| `type` | *必須* | ソースタイプ：`promphp-redis` |
| `redis_url` | *必須* | Redis接続URL |
| `prefix` | `PROMETHEUS_` | Redisキープレフィックス |
| `cache_ttl_seconds` | `0` | キャッシュTTL（秒）（0=無効） |
| `label_format` | `auto` | ラベル形式：`auto`、`json`、`base64` |
| `labels` | `{}` | すべてのメトリクスに追加するラベル |

## エンドポイント

| エンドポイント | 認証 | 説明 |
|----------------|------|------|
| `GET /metrics` | あり | Prometheusメトリクス |
| `GET /health` | なし | ヘルスチェック（常に200） |
| `GET /ready` | なし | レディネスチェック（すべてのソースが正常なら200） |

## セルフメトリクス

エクスポーターは自身のメトリクスを公開します：

```
# ソースごとのスクレイプ時間
metrics_bridge_scrape_duration_seconds{source="web-app"} 0.003

# ソースごとのスクレイプエラー
metrics_bridge_scrape_errors_total{source="web-app"} 0

# ソースの健全性（1=稼働中、0=停止）
metrics_bridge_source_up{source="web-app"} 1

# 設定されたソースの総数
metrics_bridge_sources_total 2

# エクスポーターのステータス
metrics_bridge_up 1

# ビルド情報
metrics_bridge_build_info{version="0.1.0",rust_version="1.84"} 1
```

## ビルド

```bash
# ビルド
cargo build --release

# テスト
cargo test

# リンター
cargo clippy --all-targets -- -D warnings

# Dockerイメージ
docker build -t metrics-bridge .
```

## ライセンス

MIT License - 詳細は[LICENSE](LICENSE)を参照。

## コントリビュート

歓迎します！まず[CONTRIBUTING.md](CONTRIBUTING.md)をお読みください。
