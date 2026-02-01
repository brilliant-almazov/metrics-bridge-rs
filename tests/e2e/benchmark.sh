#!/bin/sh
# Performance benchmark for metrics-bridge
set -e

BRIDGE_URL="http://metrics-bridge:9090"
ITERATIONS=1000

echo "=== Performance Benchmark ==="
echo "Target: $BRIDGE_URL/metrics"
echo "Iterations: $ITERATIONS"
echo ""

# Warmup
echo "Warming up..."
for i in $(seq 1 10); do
    curl -s -o /dev/null "$BRIDGE_URL/metrics"
done

# Benchmark
echo "Running benchmark..."
START=$(date +%s%N)

for i in $(seq 1 $ITERATIONS); do
    curl -s -o /dev/null "$BRIDGE_URL/metrics"
done

END=$(date +%s%N)

# Calculate results
DURATION_NS=$((END - START))
DURATION_MS=$((DURATION_NS / 1000000))
DURATION_S=$(echo "scale=3; $DURATION_MS / 1000" | bc)
RPS=$(echo "scale=2; $ITERATIONS / $DURATION_S" | bc)
AVG_MS=$(echo "scale=3; $DURATION_MS / $ITERATIONS" | bc)

echo ""
echo "=== Results ==="
echo "Total time: ${DURATION_S}s"
echo "Requests: $ITERATIONS"
echo "Requests/sec: $RPS"
echo "Avg latency: ${AVG_MS}ms"

# Output JSON for CI
cat > /tmp/benchmark-results.json << EOF
{
  "total_time_ms": $DURATION_MS,
  "iterations": $ITERATIONS,
  "requests_per_second": $RPS,
  "avg_latency_ms": $AVG_MS
}
EOF

echo ""
echo "Results saved to /tmp/benchmark-results.json"

# Assertions
if [ "$(echo "$AVG_MS < 10" | bc)" -eq 1 ]; then
    echo "✓ Average latency < 10ms"
else
    echo "✗ Average latency >= 10ms (FAIL)"
    exit 1
fi

if [ "$(echo "$RPS > 100" | bc)" -eq 1 ]; then
    echo "✓ Throughput > 100 req/s"
else
    echo "✗ Throughput <= 100 req/s (FAIL)"
    exit 1
fi
