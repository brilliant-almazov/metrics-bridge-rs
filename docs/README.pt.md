# metrics-bridge

[English](../README.md) | [Русский](README.ru.md) | [中文](README.zh.md) | [हिंदी](README.hi.md) | [Español](README.es.md) | **Português** | [Français](README.fr.md) | [Deutsch](README.de.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Polski](README.pl.md) | [Nederlands](README.nl.md) | [Italiano](README.it.md) | [Türkçe](README.tr.md) | [Українська](README.uk.md) | [Bahasa Indonesia](README.id.md) | [Tiếng Việt](README.vi.md) | [Svenska](README.sv.md) | [Suomi](README.fi.md)

[![Test](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/test.yml/badge.svg)](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/test.yml)
[![Release](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/release.yml/badge.svg)](https://github.com/brilliant-almazov/metrics-bridge-rs/actions/workflows/release.yml)
[![codecov](https://codecov.io/gh/brilliant-almazov/metrics-bridge-rs/graph/badge.svg)](https://codecov.io/gh/brilliant-almazov/metrics-bridge-rs)
[![Docker Image Size](https://ghcr-badge.egpl.dev/brilliant-almazov/metrics-bridge-rs/size)](https://github.com/brilliant-almazov/metrics-bridge-rs/pkgs/container/metrics-bridge-rs)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Exportador de métricas Prometheus de alto desempenho** para armazenamento Redis do [promphp/prometheus_client_php](https://github.com/promphp/prometheus_client_php). Escrito em Rust para máximo desempenho e mínimo consumo de recursos.

## Principais recursos

- **Ultra rápido** - Tempo de resposta sub-milissegundo (vs 50-200ms via PHP)
- **Alto throughput** - 12.500+ requisições/segundo com cache habilitado
- **Múltiplas fontes** - Agregação de métricas de múltiplas instâncias Redis
- **Cache flexível** - TTL de cache individual por fonte
- **Labels customizados** - Adicione labels extras às métricas (app, environment, etc.)
- **Segurança** - Basic Auth, Bearer Token, whitelist de IP, TLS
- **Compressão** - Compressão GZIP opcional
- **Auto-monitoramento** - Métricas integradas do próprio exportador
- **Imagem compacta** - Imagem Docker <10MB (baseada em scratch)
- **Compatibilidade** - Funciona com Redis, Dragonfly, Valkey, KeyDB

## Problema

Aplicações PHP usando promphp/prometheus_client_php escrevem métricas no Redis. Ler essas métricas via PHP é lento devido ao overhead do framework (~50-200ms no Symfony).

**metrics-bridge** lê métricas diretamente do Redis e as serve ao Prometheus com latência sub-milissegundo.

## Desempenho

### Com cache (recomendado)

| Métrica | Valor |
|---------|-------|
| Latência média | **0.8ms** |
| Latência P99 | 7.7ms |
| Throughput | **12.500+ req/s** |

### Sem cache

| Métrica | Valor |
|---------|-------|
| Latência média | ~19ms |
| Throughput | ~50 req/s |

## Início rápido

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

## Configuração

Crie o arquivo `config.yaml`:

```yaml
server:
  port: 9090

  # Opcional: compressão GZIP
  gzip_level: 6  # 1-9 (1=mais rápido, 9=melhor compressão)

  auth:
    type: basic  # none | basic | bearer
    username: prometheus
    password: ${METRICS_PASSWORD}  # Substituição de variáveis de ambiente

  # Opcional: whitelist de IP
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
    cache_ttl_seconds: 5  # Cache por 5 segundos
    labels:
      app: web

  - name: worker-app
    type: promphp-redis
    redis_url: ${REDIS_WORKER_URL}
    prefix: "PROMETHEUS_"
    cache_ttl_seconds: 0  # Sem cache
    labels:
      app: worker
```

### Parâmetros do servidor

| Parâmetro | Padrão | Descrição |
|-----------|--------|-----------|
| `port` | `9090` | Porta do servidor HTTP |
| `gzip_level` | - | Nível de compressão GZIP 1-9, desabilitado se não definido |
| `auth.type` | `none` | Tipo de autenticação: `none`, `basic`, `bearer` |
| `auth.username` | - | Usuário para basic auth |
| `auth.password` | - | Senha para basic auth |
| `auth.token` | - | Token para bearer auth |
| `allowed_ips` | `[]` | Whitelist de IP (CIDR), vazio = permitir todos |
| `tls.cert` | - | Caminho para o certificado TLS |
| `tls.key` | - | Caminho para a chave privada TLS |

### Parâmetros das fontes

| Parâmetro | Padrão | Descrição |
|-----------|--------|-----------|
| `name` | *obrigatório* | Identificador da fonte |
| `type` | *obrigatório* | Tipo da fonte: `promphp-redis` |
| `redis_url` | *obrigatório* | URL de conexão Redis |
| `prefix` | `PROMETHEUS_` | Prefixo das chaves Redis |
| `cache_ttl_seconds` | `0` | TTL do cache em segundos (0 = desabilitado) |
| `label_format` | `auto` | Formato dos labels: `auto`, `json`, `base64` |
| `labels` | `{}` | Labels adicionais para todas as métricas |

## Endpoints

| Endpoint | Autenticação | Descrição |
|----------|--------------|-----------|
| `GET /metrics` | Sim | Métricas Prometheus |
| `GET /health` | Não | Verificação de saúde (sempre 200) |
| `GET /ready` | Não | Verificação de prontidão (200 se todas as fontes saudáveis) |

## Métricas próprias

O exportador expõe métricas sobre si mesmo:

```
# Duração do scrape por fonte
metrics_bridge_scrape_duration_seconds{source="web-app"} 0.003

# Erros de scrape por fonte
metrics_bridge_scrape_errors_total{source="web-app"} 0

# Saúde da fonte (1=ativa, 0=inativa)
metrics_bridge_source_up{source="web-app"} 1

# Total de fontes configuradas
metrics_bridge_sources_total 2

# Status do exportador
metrics_bridge_up 1

# Informações de build
metrics_bridge_build_info{version="0.1.0",rust_version="1.84"} 1
```

## Compilação

```bash
# Compilar
cargo build --release

# Testes
cargo test

# Linter
cargo clippy --all-targets -- -D warnings

# Imagem Docker
docker build -t metrics-bridge .
```

## Licença

MIT License - veja [LICENSE](LICENSE) para detalhes.

## Contribuições

Bem-vindas! Por favor leia [CONTRIBUTING.md](CONTRIBUTING.md) primeiro.
