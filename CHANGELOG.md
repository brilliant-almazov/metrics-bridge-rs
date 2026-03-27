# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.5] - 2025-03-27

### Changed

- **Reduced memory allocations during rendering by ~47%** — renderer no longer
  clones `extra_labels` HashMap per sample; instead iterates both label maps
  directly without intermediate allocation
- **Eliminated deep cloning in cache** — `Cache<T>` now stores data behind `Arc<T>`,
  so cache hits return a cheap reference-counted pointer instead of copying the
  entire MetricFamily (~5.4 KB per source) or rendered string (~1.7 MB)

#### Benchmark (15K samples, 300 metrics, 3 sources)

| Metric | Before | After | Change |
|---|---|---|---|
| Render alloc calls (single) | 645,324 | 345,224 | **-46.5%** |
| Render alloc calls (10x) | 6,453,240 | 3,452,240 | **-46.5%** |
| Render peak memory | 2,694 KB | 2,690 KB | ~same |
| MetricFamily storage | 5,385 KB | 5,385 KB | ~same |
| Cache hit cost | full clone (~5.4 KB/source) | Arc clone (8 bytes) | **~99.9%** |

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
