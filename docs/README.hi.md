# metrics-bridge

[English](../README.md) | [Русский](README.ru.md) | [中文](README.zh.md) | **हिंदी** | [Español](README.es.md) | [Português](README.pt.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [日本語](README.ja.md) | [한국어](README.ko.md)

[![Test](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/test.yml/badge.svg)](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/test.yml)
[![Release](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/release.yml/badge.svg)](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/release.yml)
[![codecov](https://codecov.io/gh/brilliant-almazov/metrics-bridge-rs/graph/badge.svg)](https://codecov.io/gh/brilliant-almazov/metrics-bridge-rs)
[![Docker Image Size](https://ghcr-badge.egpl.dev/brilliant-almazov/metrics-bridge-rs/size)](https://github.com/brilliant-almazov/metrics-bridge-rs/pkgs/container/metrics-bridge-rs)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**उच्च-प्रदर्शन Prometheus मेट्रिक्स एक्सपोर्टर** [promphp/prometheus_client_php](https://github.com/promphp/prometheus_client_php) Redis स्टोरेज के लिए। अधिकतम प्रदर्शन और न्यूनतम संसाधन उपयोग के लिए Rust में लिखा गया।

## मुख्य विशेषताएं

- **अति-तेज** - सब-मिलीसेकंड रिस्पॉन्स टाइम (PHP के 50-200ms की तुलना में)
- **उच्च थ्रूपुट** - कैशिंग के साथ 12,500+ अनुरोध/सेकंड
- **एकाधिक स्रोत** - कई Redis इंस्टेंस से मेट्रिक्स एकत्र करें
- **लचीला कैशिंग** - प्रत्येक स्रोत के लिए अलग कैश TTL
- **कस्टम लेबल** - मेट्रिक्स में अतिरिक्त लेबल जोड़ें (app, environment आदि)
- **सुरक्षा** - Basic Auth, Bearer Token, IP व्हाइटलिस्ट, TLS
- **संपीड़न** - वैकल्पिक GZIP संपीड़न
- **स्व-मॉनिटरिंग** - एक्सपोर्टर की अपनी मेट्रिक्स
- **छोटा इमेज** - 10MB से कम Docker इमेज (scratch-based)
- **संगतता** - Redis, Dragonfly, Valkey, KeyDB के साथ काम करता है

## समस्या

promphp/prometheus_client_php का उपयोग करने वाले PHP एप्लिकेशन Redis में मेट्रिक्स लिखते हैं। PHP के माध्यम से इन मेट्रिक्स को पढ़ना फ्रेमवर्क ओवरहेड के कारण धीमा है (~50-200ms Symfony में)।

**metrics-bridge** सीधे Redis से मेट्रिक्स पढ़ता है और उन्हें सब-मिलीसेकंड लेटेंसी के साथ Prometheus को देता है।

## प्रदर्शन

### कैशिंग के साथ (अनुशंसित)

| मेट्रिक | मान |
|---------|-----|
| औसत लेटेंसी | **0.8ms** |
| P99 लेटेंसी | 7.7ms |
| थ्रूपुट | **12,500+ req/s** |

### बिना कैशिंग

| मेट्रिक | मान |
|---------|-----|
| औसत लेटेंसी | ~19ms |
| थ्रूपुट | ~50 req/s |

## त्वरित शुरुआत

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

## कॉन्फ़िगरेशन

`config.yaml` फ़ाइल बनाएं:

```yaml
server:
  port: 9090

  # वैकल्पिक: GZIP संपीड़न
  gzip_level: 6  # 1-9 (1=सबसे तेज, 9=सर्वश्रेष्ठ संपीड़न)

  auth:
    type: basic  # none | basic | bearer
    username: prometheus
    password: ${METRICS_PASSWORD}  # पर्यावरण चर प्रतिस्थापन

  # वैकल्पिक: IP व्हाइटलिस्ट
  allowed_ips:
    - 10.0.0.0/8
    - 192.168.0.0/16

  # वैकल्पिक: TLS
  tls:
    cert: /path/to/cert.pem
    key: /path/to/key.pem

sources:
  - name: web-app
    type: promphp-redis
    redis_url: ${REDIS_WEB_URL}
    prefix: "PROMETHEUS_"
    cache_ttl_seconds: 5  # 5 सेकंड के लिए कैश
    labels:
      app: web

  - name: worker-app
    type: promphp-redis
    redis_url: ${REDIS_WORKER_URL}
    prefix: "PROMETHEUS_"
    cache_ttl_seconds: 0  # कोई कैशिंग नहीं
    labels:
      app: worker
```

### सर्वर पैरामीटर

| पैरामीटर | डिफ़ॉल्ट | विवरण |
|----------|---------|--------|
| `port` | `9090` | HTTP सर्वर पोर्ट |
| `gzip_level` | - | GZIP संपीड़न स्तर 1-9, सेट न होने पर अक्षम |
| `auth.type` | `none` | प्रमाणीकरण प्रकार: `none`, `basic`, `bearer` |
| `auth.username` | - | Basic auth के लिए उपयोगकर्ता नाम |
| `auth.password` | - | Basic auth के लिए पासवर्ड |
| `auth.token` | - | Bearer auth के लिए टोकन |
| `allowed_ips` | `[]` | IP व्हाइटलिस्ट (CIDR), खाली = सभी अनुमत |
| `tls.cert` | - | TLS प्रमाणपत्र पथ |
| `tls.key` | - | TLS निजी कुंजी पथ |

### स्रोत पैरामीटर

| पैरामीटर | डिफ़ॉल्ट | विवरण |
|----------|---------|--------|
| `name` | *आवश्यक* | स्रोत पहचानकर्ता |
| `type` | *आवश्यक* | स्रोत प्रकार: `promphp-redis` |
| `redis_url` | *आवश्यक* | Redis कनेक्शन URL |
| `prefix` | `PROMETHEUS_` | Redis कुंजी उपसर्ग |
| `cache_ttl_seconds` | `0` | कैश TTL सेकंड में (0 = अक्षम) |
| `label_format` | `auto` | लेबल प्रारूप: `auto`, `json`, `base64` |
| `labels` | `{}` | सभी मेट्रिक्स में जोड़ने के लिए अतिरिक्त लेबल |

## एंडपॉइंट्स

| एंडपॉइंट | प्रमाणीकरण | विवरण |
|----------|------------|--------|
| `GET /metrics` | हां | Prometheus मेट्रिक्स |
| `GET /health` | नहीं | स्वास्थ्य जांच (हमेशा 200) |
| `GET /ready` | नहीं | तत्परता जांच (200 यदि सभी स्रोत स्वस्थ) |

## स्व-मेट्रिक्स

एक्सपोर्टर अपनी मेट्रिक्स प्रदान करता है:

```
# प्रति स्रोत स्क्रैप अवधि
metrics_bridge_scrape_duration_seconds{source="web-app"} 0.003

# प्रति स्रोत स्क्रैप त्रुटियां
metrics_bridge_scrape_errors_total{source="web-app"} 0

# स्रोत स्वास्थ्य (1=ऊपर, 0=नीचे)
metrics_bridge_source_up{source="web-app"} 1

# कुल कॉन्फ़िगर किए गए स्रोत
metrics_bridge_sources_total 2

# एक्सपोर्टर स्थिति
metrics_bridge_up 1

# बिल्ड जानकारी
metrics_bridge_build_info{version="0.1.0",rust_version="1.84"} 1
```

## बिल्डिंग

```bash
# बिल्ड
cargo build --release

# टेस्ट
cargo test

# लिंटर
cargo clippy --all-targets -- -D warnings

# Docker इमेज
docker build -t metrics-bridge .
```

## लाइसेंस

MIT License - विवरण के लिए [LICENSE](LICENSE) देखें।

## योगदान

स्वागत है! कृपया पहले [CONTRIBUTING.md](CONTRIBUTING.md) पढ़ें।
