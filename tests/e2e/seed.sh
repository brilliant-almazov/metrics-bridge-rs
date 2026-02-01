#!/bin/sh
# Seed test data to all Redis-compatible stores in promphp format
# Generates a large number of metrics for realistic testing
set -e

# Number of unique metric variations to generate
NUM_COUNTERS=50
NUM_GAUGES=30
NUM_HISTOGRAMS=20

# Function to seed data to a Redis instance
seed_redis() {
    HOST=$1
    PORT=$2
    echo "Seeding data to $HOST:$PORT..."

    # Generate many counter metrics
    for i in $(seq 1 $NUM_COUNTERS); do
        METRIC_NAME="http_requests_total_${i}"
        redis-cli -h "$HOST" -p "$PORT" SADD "PROMETHEUS_:COUNTER_METRIC_KEYS" "$METRIC_NAME" > /dev/null

        # Multiple label combinations per metric
        for method in GET POST PUT DELETE; do
            for status in 200 201 400 404 500; do
                VALUE=$((RANDOM % 10000))
                LABELS=$(echo -n "[\"$method\",\"$status\"]" | base64)
                redis-cli -h "$HOST" -p "$PORT" HSET "PROMETHEUS_:counter:$METRIC_NAME" \
                    "__meta" "{\"name\":\"$METRIC_NAME\",\"help\":\"HTTP requests counter $i\",\"type\":\"counter\",\"labelNames\":[\"method\",\"status\"]}" \
                    "$LABELS" "$VALUE" > /dev/null
            done
        done
    done
    echo "  - $NUM_COUNTERS counters with $((NUM_COUNTERS * 20)) samples"

    # Generate many gauge metrics
    for i in $(seq 1 $NUM_GAUGES); do
        METRIC_NAME="process_metric_${i}"
        redis-cli -h "$HOST" -p "$PORT" SADD "PROMETHEUS_:GAUGE_METRIC_KEYS" "$METRIC_NAME" > /dev/null

        for type in heap stack rss vms; do
            VALUE=$((RANDOM % 100000000))
            LABELS=$(echo -n "[\"$type\"]" | base64)
            redis-cli -h "$HOST" -p "$PORT" HSET "PROMETHEUS_:gauge:$METRIC_NAME" \
                "__meta" "{\"name\":\"$METRIC_NAME\",\"help\":\"Process metric $i\",\"type\":\"gauge\",\"labelNames\":[\"type\"]}" \
                "$LABELS" "$VALUE" > /dev/null
        done
    done
    echo "  - $NUM_GAUGES gauges with $((NUM_GAUGES * 4)) samples"

    # Generate histogram metrics
    for i in $(seq 1 $NUM_HISTOGRAMS); do
        METRIC_NAME="request_duration_${i}"
        redis-cli -h "$HOST" -p "$PORT" SADD "PROMETHEUS_:HISTOGRAM_METRIC_KEYS" "$METRIC_NAME" > /dev/null

        for endpoint in /api/users /api/orders /api/products /api/health /api/metrics; do
            SUM=$(echo "scale=2; $RANDOM / 100" | bc)
            COUNT=$((RANDOM % 5000 + 100))
            LABELS=$(echo -n "[\"$endpoint\"]" | base64)
            redis-cli -h "$HOST" -p "$PORT" HSET "PROMETHEUS_:histogram:$METRIC_NAME" \
                "__meta" "{\"name\":\"$METRIC_NAME\",\"help\":\"Request duration $i\",\"type\":\"histogram\",\"labelNames\":[\"endpoint\"],\"buckets\":[0.005,0.01,0.025,0.05,0.1,0.25,0.5,1,2.5,5,10]}" \
                "$LABELS" "{\"sum\":$SUM,\"count\":$COUNT,\"buckets\":{\"0.005\":$((COUNT/10)),\"0.01\":$((COUNT/5)),\"0.025\":$((COUNT/3)),\"0.05\":$((COUNT/2)),\"0.1\":$((COUNT*6/10)),\"0.25\":$((COUNT*7/10)),\"0.5\":$((COUNT*8/10)),\"1\":$((COUNT*9/10)),\"2.5\":$((COUNT*95/100)),\"5\":$COUNT,\"10\":$COUNT}}" > /dev/null
        done
    done
    echo "  - $NUM_HISTOGRAMS histograms with $((NUM_HISTOGRAMS * 5)) samples"

    TOTAL_METRICS=$((NUM_COUNTERS + NUM_GAUGES + NUM_HISTOGRAMS))
    TOTAL_SAMPLES=$((NUM_COUNTERS * 20 + NUM_GAUGES * 4 + NUM_HISTOGRAMS * 5))
    echo "  Total: $TOTAL_METRICS metrics, $TOTAL_SAMPLES samples"
}

# Seed all stores
seed_redis redis 6379
seed_redis dragonfly 6379
seed_redis valkey 6379
seed_redis keydb 6379

echo ""
echo "All stores seeded successfully!"
echo "Total metrics per store: $((NUM_COUNTERS + NUM_GAUGES + NUM_HISTOGRAMS))"
