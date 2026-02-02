#!/bin/bash
# Seed a large dataset for load testing
set -e

REDIS_CLI="docker exec redis_prometheus.cgw redis-cli"

echo "Seeding load test data..."

# Clear old test data
$REDIS_CLI KEYS "LOADTEST_:*" | xargs -r $REDIS_CLI DEL 2>/dev/null || true

# Generate 500 counter metrics
echo "Creating 500 counter metrics..."
for i in $(seq 1 500); do
    METRIC_NAME="loadtest_http_requests_${i}"
    $REDIS_CLI SADD "LOADTEST_:COUNTER_METRIC_KEYS" "$METRIC_NAME" > /dev/null

    # 20 label combinations per metric
    for method in GET POST PUT DELETE PATCH; do
        for status in 200 201 400 404; do
            VALUE=$((RANDOM % 100000))
            LABELS=$(echo -n "[\"$method\",\"$status\"]" | base64)
            $REDIS_CLI HSET "LOADTEST_:counter:$METRIC_NAME" \
                "__meta" "{\"name\":\"$METRIC_NAME\",\"help\":\"HTTP requests $i\",\"type\":\"counter\",\"labelNames\":[\"method\",\"status\"]}" \
                "$LABELS" "$VALUE" > /dev/null
        done
    done

    if [ $((i % 100)) -eq 0 ]; then
        echo "  Created $i counters..."
    fi
done

# Generate 200 gauge metrics
echo "Creating 200 gauge metrics..."
for i in $(seq 1 200); do
    METRIC_NAME="loadtest_process_metric_${i}"
    $REDIS_CLI SADD "LOADTEST_:GAUGE_METRIC_KEYS" "$METRIC_NAME" > /dev/null

    for type in heap stack rss vms resident shared code data; do
        VALUE=$((RANDOM % 1000000000))
        LABELS=$(echo -n "[\"$type\"]" | base64)
        $REDIS_CLI HSET "LOADTEST_:gauge:$METRIC_NAME" \
            "__meta" "{\"name\":\"$METRIC_NAME\",\"help\":\"Process metric $i\",\"type\":\"gauge\",\"labelNames\":[\"type\"]}" \
            "$LABELS" "$VALUE" > /dev/null
    done

    if [ $((i % 50)) -eq 0 ]; then
        echo "  Created $i gauges..."
    fi
done

# Generate 100 histogram metrics
echo "Creating 100 histogram metrics..."
for i in $(seq 1 100); do
    METRIC_NAME="loadtest_request_duration_${i}"
    $REDIS_CLI SADD "LOADTEST_:HISTOGRAM_METRIC_KEYS" "$METRIC_NAME" > /dev/null

    for endpoint in /api/v1/users /api/v1/orders /api/v1/products /api/v1/auth /api/v1/metrics /api/v2/users /api/v2/orders /api/v2/products; do
        SUM=$(echo "scale=2; $RANDOM / 100" | bc)
        COUNT=$((RANDOM % 10000 + 1000))
        LABELS=$(echo -n "[\"$endpoint\"]" | base64)
        $REDIS_CLI HSET "LOADTEST_:histogram:$METRIC_NAME" \
            "__meta" "{\"name\":\"$METRIC_NAME\",\"help\":\"Request duration $i\",\"type\":\"histogram\",\"labelNames\":[\"endpoint\"],\"buckets\":[0.005,0.01,0.025,0.05,0.1,0.25,0.5,1,2.5,5,10]}" \
            "$LABELS" "{\"sum\":$SUM,\"count\":$COUNT,\"buckets\":{\"0.005\":$((COUNT/10)),\"0.01\":$((COUNT/5)),\"0.025\":$((COUNT/3)),\"0.05\":$((COUNT/2)),\"0.1\":$((COUNT*6/10)),\"0.25\":$((COUNT*7/10)),\"0.5\":$((COUNT*8/10)),\"1\":$((COUNT*9/10)),\"2.5\":$((COUNT*95/100)),\"5\":$COUNT,\"10\":$COUNT}}" > /dev/null
    done

    if [ $((i % 25)) -eq 0 ]; then
        echo "  Created $i histograms..."
    fi
done

# Print summary
TOTAL_COUNTERS=500
TOTAL_GAUGES=200
TOTAL_HISTOGRAMS=100
TOTAL_SAMPLES=$((TOTAL_COUNTERS * 20 + TOTAL_GAUGES * 8 + TOTAL_HISTOGRAMS * 8))

echo ""
echo "=== Load Test Data Seeded ==="
echo "Counters:   $TOTAL_COUNTERS metrics × 20 samples = $((TOTAL_COUNTERS * 20)) samples"
echo "Gauges:     $TOTAL_GAUGES metrics × 8 samples = $((TOTAL_GAUGES * 8)) samples"
echo "Histograms: $TOTAL_HISTOGRAMS metrics × 8 samples = $((TOTAL_HISTOGRAMS * 8)) samples"
echo "Total:      $((TOTAL_COUNTERS + TOTAL_GAUGES + TOTAL_HISTOGRAMS)) metrics, $TOTAL_SAMPLES samples"
