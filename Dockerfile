# Build stage
FROM rust:1.85-slim AS builder
WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y libsqlite3-dev libssl-dev pkg-config && rm -rf /var/lib/apt/lists/*

# Copy manifests and source
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build release binary
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y libsqlite3-0 libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/sparkling /usr/local/bin/

CMD ["sparkling"]
