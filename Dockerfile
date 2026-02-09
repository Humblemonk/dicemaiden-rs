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
FROM registry.access.redhat.com/ubi${UBI_VERSION}/ubi-minimal:9.7

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

# Create user to run container as non-root
RUN useradd -m -u 1000 -s /bin/sh dicemaiden

# Set up application directory and data volume
WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /app/target/release/dicemaiden-rs /usr/local/bin/dicemaiden-rs
RUN chmod 755 /usr/local/bin/dicemaiden-rs

# Verify the binary exists and is executable
RUN test -x /usr/local/bin/dicemaiden-rs && echo "Binary xists and is executable"

# Set ownership of application directory to non-root user
RUN chown -R dicemaiden:dicemaiden /app

# Switch to non-root user
USER dicemaiden

# Verify binary is executable by dicemaiden user
RUN test -x /usr/local/bin/dicemaiden-rs && \
    echo "User 'dicemaiden' can execute the binary" || \
    (echo "ERROR: User 'dicemaiden' cannot execute binary!" && exit 1)

# Verify dicemaiden can access the working directory
RUN test -w /app && test -r /app && \
    echo "User 'dicemaiden' has read/write access to /app" || \
    (echo "ERROR: User 'dicemaiden' cannot access /app!" && exit 1)

# This allows Docker/Kubernetes to monitor container health
HEALTHCHECK --interval=30s --timeout=3s --start-period=60s --retries=3 \
    CMD pgrep -f dicemaiden-rs || exit 1

# Set the entrypoint
ENTRYPOINT ["/usr/local/bin/dicemaiden-rs"]
