# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2025-02-01

### Added

- Initial release
- promphp Redis source support (counters, gauges, histograms)
- Multiple source aggregation with parallel collection
- Authentication: Basic, Bearer, IP whitelist
- Self-monitoring metrics
- Health and readiness endpoints
- Environment variable substitution in config
- Docker image with scratch base (<10MB)
- GitHub Actions CI/CD
- GCP Cloud Build support
- E2E tests with Redis, Dragonfly, Valkey, KeyDB
