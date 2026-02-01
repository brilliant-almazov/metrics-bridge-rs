#!/bin/sh
# E2E test runner for metrics-bridge
set -e

BRIDGE_URL="http://metrics-bridge:9090"
PASS=0
FAIL=0

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

assert_contains() {
    RESPONSE="$1"
    EXPECTED="$2"
    TEST_NAME="$3"

    if echo "$RESPONSE" | grep -q "$EXPECTED"; then
        echo "${GREEN}✓${NC} $TEST_NAME"
        PASS=$((PASS + 1))
    else
        echo "${RED}✗${NC} $TEST_NAME"
        echo "  Expected to contain: $EXPECTED"
        FAIL=$((FAIL + 1))
    fi
}

assert_status() {
    ACTUAL="$1"
    EXPECTED="$2"
    TEST_NAME="$3"

    if [ "$ACTUAL" = "$EXPECTED" ]; then
        echo "${GREEN}✓${NC} $TEST_NAME"
        PASS=$((PASS + 1))
    else
        echo "${RED}✗${NC} $TEST_NAME"
        echo "  Expected: $EXPECTED, Got: $ACTUAL"
        FAIL=$((FAIL + 1))
    fi
}

echo "=== E2E Tests for metrics-bridge ==="
echo ""

# Test 1: Health endpoint
echo "--- Health Check ---"
STATUS=$(curl -s -o /dev/null -w "%{http_code}" "$BRIDGE_URL/health")
assert_status "$STATUS" "200" "GET /health returns 200"

# Test 2: Ready endpoint
echo ""
echo "--- Readiness Check ---"
STATUS=$(curl -s -o /dev/null -w "%{http_code}" "$BRIDGE_URL/ready")
assert_status "$STATUS" "200" "GET /ready returns 200"

READY_BODY=$(curl -s "$BRIDGE_URL/ready")
assert_contains "$READY_BODY" "redis-test: ok" "Redis source is ready"
assert_contains "$READY_BODY" "dragonfly-test: ok" "Dragonfly source is ready"
assert_contains "$READY_BODY" "valkey-test: ok" "Valkey source is ready"
assert_contains "$READY_BODY" "keydb-test: ok" "KeyDB source is ready"

# Test 3: Metrics endpoint
echo ""
echo "--- Metrics Check ---"
STATUS=$(curl -s -o /dev/null -w "%{http_code}" "$BRIDGE_URL/metrics")
assert_status "$STATUS" "200" "GET /metrics returns 200"

METRICS=$(curl -s "$BRIDGE_URL/metrics")

# Check counter metrics from all stores
assert_contains "$METRICS" 'http_requests_total{' "Counter metric present"
assert_contains "$METRICS" 'store="redis"' "Redis store label present"
assert_contains "$METRICS" 'store="dragonfly"' "Dragonfly store label present"
assert_contains "$METRICS" 'store="valkey"' "Valkey store label present"
assert_contains "$METRICS" 'store="keydb"' "KeyDB store label present"

# Check gauge metrics
assert_contains "$METRICS" 'process_memory_bytes{' "Gauge metric present"

# Check histogram metrics
assert_contains "$METRICS" 'http_request_duration_seconds_bucket{' "Histogram bucket metric present"
assert_contains "$METRICS" 'http_request_duration_seconds_sum{' "Histogram sum metric present"
assert_contains "$METRICS" 'http_request_duration_seconds_count{' "Histogram count metric present"

# Check self-metrics
assert_contains "$METRICS" 'metrics_bridge_up 1' "Self-metric: up"
assert_contains "$METRICS" 'metrics_bridge_sources_total 4' "Self-metric: sources_total"
assert_contains "$METRICS" 'metrics_bridge_scrape_duration_seconds' "Self-metric: scrape_duration"

# Test 4: Response time (should be <10ms)
echo ""
echo "--- Performance Check ---"
TIME=$(curl -s -o /dev/null -w "%{time_total}" "$BRIDGE_URL/metrics")
TIME_MS=$(echo "$TIME * 1000" | bc)
if [ "$(echo "$TIME_MS < 100" | bc)" -eq 1 ]; then
    echo "${GREEN}✓${NC} Response time ${TIME_MS}ms < 100ms"
    PASS=$((PASS + 1))
else
    echo "${RED}✗${NC} Response time ${TIME_MS}ms >= 100ms"
    FAIL=$((FAIL + 1))
fi

# Summary
echo ""
echo "=== Summary ==="
echo "${GREEN}Passed: $PASS${NC}"
echo "${RED}Failed: $FAIL${NC}"

if [ $FAIL -gt 0 ]; then
    exit 1
fi

echo ""
echo "All tests passed!"
