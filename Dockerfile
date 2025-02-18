# Stage 1: Build the Rust binary
FROM rust:slim AS builder

# Set working directory
WORKDIR /app

# Install dependencies for building
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Copy dependency manifests
COPY Cargo.toml Cargo.lock ./

# Cache dependencies
RUN cargo fetch --locked

# Copy source code
COPY src ./src

# Copy configuration files
COPY log4rs.yml ./
COPY .env ./

# Build release binary
RUN cargo build --release --locked

# Stage 2: Runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y libssl3 ca-certificates && \
    update-ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy compiled binary from builder
COPY --from=builder /app/target/release/glebus_vpn_bot /usr/local/bin/

# Copy configuration files
COPY --from=builder /app/log4rs.yml /home/botuser/log4rs.yml
COPY --from=builder /app/.env /home/botuser/.env

# Create non-root user and set permissions
RUN useradd -m botuser && \
    chown -R botuser:botuser /home/botuser && \
    chmod +x /usr/local/bin/glebus_vpn_bot

# Switch to non-root user
USER botuser

# Set working directory
WORKDIR /home/botuser

# Set entrypoint
ENTRYPOINT ["/usr/local/bin/glebus_vpn_bot"]