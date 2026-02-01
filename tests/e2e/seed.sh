#!/bin/sh
# Seed test data to all Redis-compatible stores in promphp format
set -e

# Function to seed data to a Redis instance
seed_redis() {
    HOST=$1
    PORT=$2
    echo "Seeding data to $HOST:$PORT..."

    # Counter metric
    redis-cli -h "$HOST" -p "$PORT" SADD "PROMETHEUS_:COUNTER_METRIC_KEYS" "http_requests_total"
    redis-cli -h "$HOST" -p "$PORT" HSET "PROMETHEUS_:counter:http_requests_total" \
        "__meta" '{"name":"http_requests_total","help":"Total HTTP requests","type":"counter","labelNames":["method","status"]}' \
        "$(echo -n '["GET","200"]' | base64)" "1500" \
        "$(echo -n '["GET","404"]' | base64)" "25" \
        "$(echo -n '["POST","200"]' | base64)" "340" \
        "$(echo -n '["POST","500"]' | base64)" "5"

    # Gauge metric
    redis-cli -h "$HOST" -p "$PORT" SADD "PROMETHEUS_:GAUGE_METRIC_KEYS" "process_memory_bytes"
    redis-cli -h "$HOST" -p "$PORT" HSET "PROMETHEUS_:gauge:process_memory_bytes" \
        "__meta" '{"name":"process_memory_bytes","help":"Process memory usage in bytes","type":"gauge","labelNames":["type"]}' \
        "$(echo -n '["heap"]' | base64)" "52428800" \
        "$(echo -n '["stack"]' | base64)" "1048576"

    # Histogram metric
    redis-cli -h "$HOST" -p "$PORT" SADD "PROMETHEUS_:HISTOGRAM_METRIC_KEYS" "http_request_duration_seconds"
    redis-cli -h "$HOST" -p "$PORT" HSET "PROMETHEUS_:histogram:http_request_duration_seconds" \
        "__meta" '{"name":"http_request_duration_seconds","help":"HTTP request duration in seconds","type":"histogram","labelNames":["endpoint"],"buckets":[0.005,0.01,0.025,0.05,0.1,0.25,0.5,1,2.5,5,10]}' \
        "$(echo -n '["/api/users"]' | base64)" '{"sum":45.5,"count":1000,"buckets":{"0.005":100,"0.01":250,"0.025":500,"0.05":700,"0.1":850,"0.25":950,"0.5":980,"1":995,"2.5":998,"5":1000,"10":1000}}'

    echo "Seeded $HOST:$PORT successfully"
}

# Seed all stores
seed_redis redis 6379
seed_redis dragonfly 6379
seed_redis valkey 6379
seed_redis keydb 6379

echo "All stores seeded successfully!"
