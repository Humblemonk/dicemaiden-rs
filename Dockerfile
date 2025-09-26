# Build arguments for version pinning
ARG RUST_VERSION=1.90.0
ARG UBI_VERSION=9

# Use official Rust image for building (faster, more reliable)
FROM rust:${RUST_VERSION} AS builder

# Add metadata labels
LABEL org.opencontainers.image.title="Dice Maiden"
LABEL org.opencontainers.image.description="Discord Dice bot"
LABEL org.opencontainers.image.source="https://github.com/Humblemonk/dicemaiden-rs"

# Install build dependencies in single layer
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        pkg-config \
        libssl-dev && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*

# Set environment variables for linking
ENV OPENSSL_STATIC=1
ENV PKG_CONFIG_ALLOW_CROSS=1

# Set up the working directory
WORKDIR /app

# Copy manifest files first for better caching
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (this layer will be cached)
# Use --lib to avoid building the binary, just dependencies
RUN cargo build --release --locked --lib 2>/dev/null || true && rm -rf src

# Copy source code
COPY src ./src

# Build the application
RUN cargo build --release --locked

# Runtime stage - UBI Minimal
FROM registry.access.redhat.com/ubi${UBI_VERSION}/ubi-minimal:latest

# Add metadata
LABEL org.opencontainers.image.title="Dice Maiden Runtime"
LABEL org.opencontainers.image.description="Runtime image for Dice Maiden Discord bot"

# Install runtime dependencies
RUN microdnf update -y && \
    microdnf install -y \
        openssl-libs \
        ca-certificates \
        tzdata \
        sqlite-libs && \
    microdnf clean all && \
    rm -rf /var/cache/yum

# Set up application directory and data volume
WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /app/target/release/dicemaiden-rs /usr/local/bin/dicemaiden-rs
RUN chmod +x /usr/local/bin/dicemaiden-rs

# Verify the binary exists and is executable
RUN test -x /usr/local/bin/dicemaiden-rs && echo "Binary ready for execution"

# Set the entrypoint
ENTRYPOINT ["/usr/local/bin/dicemaiden-rs"]
