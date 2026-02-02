# Build stage
FROM rust:1.85-alpine AS builder

RUN apk add --no-cache musl-dev

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock* ./

# Create dummy src to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src

# Copy actual source
COPY src ./src

# Build for real
RUN touch src/main.rs && cargo build --release

# Runtime stage - scratch for minimal image
FROM scratch

# Copy binary
COPY --from=builder /app/target/release/metrics-bridge /metrics-bridge

# Copy CA certificates for HTTPS connections
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

# Default port
EXPOSE 9090

# Run
ENTRYPOINT ["/metrics-bridge"]
