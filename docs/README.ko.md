# metrics-bridge

[English](../README.md) | [Русский](README.ru.md) | [中文](README.zh.md) | [हिंदी](README.hi.md) | [Español](README.es.md) | [Português](README.pt.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [日本語](README.ja.md) | **한국어**

[![Test](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/test.yml/badge.svg)](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/test.yml)
[![Release](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/release.yml/badge.svg)](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/release.yml)
[![codecov](https://codecov.io/gh/brilliant-almazov/metrics-bridge-rs/graph/badge.svg)](https://codecov.io/gh/brilliant-almazov/metrics-bridge-rs)
[![Docker Image Size](https://ghcr-badge.egpl.dev/brilliant-almazov/metrics-bridge-rs/size)](https://github.com/brilliant-almazov/metrics-bridge-rs/pkgs/container/metrics-bridge-rs)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**고성능 Prometheus 메트릭 익스포터** - [promphp/prometheus_client_php](https://github.com/promphp/prometheus_client_php) Redis 저장소용. 최대 성능과 최소 리소스 소비를 위해 Rust로 작성되었습니다.

## 주요 기능

- **초고속** - 서브밀리초 응답 시간 (PHP의 50-200ms 대비)
- **높은 처리량** - 캐시 활성화 시 12,500+ 요청/초
- **다중 소스** - 여러 Redis 인스턴스에서 메트릭 집계
- **유연한 캐싱** - 소스별 개별 캐시 TTL
- **커스텀 레이블** - 메트릭에 추가 레이블 부여 (app, environment 등)
- **보안** - Basic Auth, Bearer Token, IP 화이트리스트, TLS
- **압축** - 선택적 GZIP 압축
- **셀프 모니터링** - 익스포터 자체의 내장 메트릭
- **컴팩트 이미지** - 10MB 미만의 Docker 이미지 (scratch 기반)
- **호환성** - Redis, Dragonfly, Valkey, KeyDB 지원

## 문제점

promphp/prometheus_client_php를 사용하는 PHP 애플리케이션은 Redis에 메트릭을 씁니다. PHP를 통해 이러한 메트릭을 읽는 것은 프레임워크 오버헤드로 인해 느립니다 (Symfony에서 ~50-200ms).

**metrics-bridge**는 Redis에서 직접 메트릭을 읽고 서브밀리초 지연 시간으로 Prometheus에 제공합니다.

## 성능

### 캐시 사용 시 (권장)

| 메트릭 | 값 |
|--------|-----|
| 평균 지연 시간 | **0.8ms** |
| P99 지연 시간 | 7.7ms |
| 처리량 | **12,500+ req/s** |

### 캐시 미사용 시

| 메트릭 | 값 |
|--------|-----|
| 평균 지연 시간 | ~19ms |
| 처리량 | ~50 req/s |

## 빠른 시작

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

## 설정

`config.yaml` 파일 생성:

```yaml
server:
  port: 9090

  # 선택사항: GZIP 압축
  gzip_level: 6  # 1-9 (1=가장 빠름, 9=최고 압축)

  auth:
    type: basic  # none | basic | bearer
    username: prometheus
    password: ${METRICS_PASSWORD}  # 환경 변수 치환

  # 선택사항: IP 화이트리스트
  allowed_ips:
    - 10.0.0.0/8
    - 192.168.0.0/16

  # 선택사항: TLS
  tls:
    cert: /path/to/cert.pem
    key: /path/to/key.pem

sources:
  - name: web-app
    type: promphp-redis
    redis_url: ${REDIS_WEB_URL}
    prefix: "PROMETHEUS_"
    cache_ttl_seconds: 5  # 5초간 캐시
    labels:
      app: web

  - name: worker-app
    type: promphp-redis
    redis_url: ${REDIS_WORKER_URL}
    prefix: "PROMETHEUS_"
    cache_ttl_seconds: 0  # 캐시 없음
    labels:
      app: worker
```

### 서버 파라미터

| 파라미터 | 기본값 | 설명 |
|----------|--------|------|
| `port` | `9090` | HTTP 서버 포트 |
| `gzip_level` | - | GZIP 압축 레벨 1-9, 미설정 시 비활성화 |
| `auth.type` | `none` | 인증 유형: `none`, `basic`, `bearer` |
| `auth.username` | - | Basic 인증 사용자명 |
| `auth.password` | - | Basic 인증 비밀번호 |
| `auth.token` | - | Bearer 인증 토큰 |
| `allowed_ips` | `[]` | IP 화이트리스트 (CIDR), 비어있으면 모두 허용 |
| `tls.cert` | - | TLS 인증서 경로 |
| `tls.key` | - | TLS 개인 키 경로 |

### 소스 파라미터

| 파라미터 | 기본값 | 설명 |
|----------|--------|------|
| `name` | *필수* | 소스 식별자 |
| `type` | *필수* | 소스 유형: `promphp-redis` |
| `redis_url` | *필수* | Redis 연결 URL |
| `prefix` | `PROMETHEUS_` | Redis 키 접두사 |
| `cache_ttl_seconds` | `0` | 캐시 TTL (초) (0 = 비활성화) |
| `label_format` | `auto` | 레이블 형식: `auto`, `json`, `base64` |
| `labels` | `{}` | 모든 메트릭에 추가할 레이블 |

## 엔드포인트

| 엔드포인트 | 인증 | 설명 |
|------------|------|------|
| `GET /metrics` | 예 | Prometheus 메트릭 |
| `GET /health` | 아니오 | 헬스 체크 (항상 200) |
| `GET /ready` | 아니오 | 레디니스 체크 (모든 소스가 정상이면 200) |

## 셀프 메트릭

익스포터는 자체 메트릭을 노출합니다:

```
# 소스별 스크레이프 시간
metrics_bridge_scrape_duration_seconds{source="web-app"} 0.003

# 소스별 스크레이프 오류
metrics_bridge_scrape_errors_total{source="web-app"} 0

# 소스 상태 (1=정상, 0=비정상)
metrics_bridge_source_up{source="web-app"} 1

# 설정된 소스 총 개수
metrics_bridge_sources_total 2

# 익스포터 상태
metrics_bridge_up 1

# 빌드 정보
metrics_bridge_build_info{version="0.1.0",rust_version="1.84"} 1
```

## 빌드

```bash
# 빌드
cargo build --release

# 테스트
cargo test

# 린터
cargo clippy --all-targets -- -D warnings

# Docker 이미지
docker build -t metrics-bridge .
```

## 라이선스

MIT License - 자세한 내용은 [LICENSE](LICENSE)를 참조하세요.

## 기여

환영합니다! 먼저 [CONTRIBUTING.md](CONTRIBUTING.md)를 읽어주세요.
