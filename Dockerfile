# Build arguments for version pinning
ARG RUST_VERSION=1.90.0
ARG UBI_VERSION=9.7

# ---------- Build stage ----------
FROM rust:${RUST_VERSION} AS builder

WORKDIR /app

# Prime the dependency cache. Both dummy targets are required: the crate has an
# autodiscovered src/lib.rs, so a main.rs-only stub cannot build the real target set.
# `cargo clean -p` drops the stub's own artifacts and fingerprints while leaving every
# dependency compiled; a plain rm of target/release/deps misses libdicemaiden_rs-*.rlib
# and all of .fingerprint/, which silently links the stub into the real binary.
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    echo "" > src/lib.rs && \
    cargo build --release --locked && \
    cargo clean --release -p dicemaiden-rs && \
    rm -rf src

# COPY preserves the build context's mtimes, which are older than the stub build above.
# Without the touch, cargo's fingerprint check treats the real sources as fresh and
# reuses the stub. Do not remove this.
COPY src ./src
RUN find src -name '*.rs' -exec touch {} + && \
    cargo build --release --locked && \
    ldd target/release/dicemaiden-rs

# ---------- Runtime stage ----------
FROM registry.access.redhat.com/ubi9/ubi-minimal:${UBI_VERSION}

LABEL org.opencontainers.image.title="Dice Maiden" \
      org.opencontainers.image.description="Discord dice bot" \
      org.opencontainers.image.source="https://github.com/Humblemonk/dicemaiden-rs"

# TLS is rustls end to end (serenity rustls_backend, sqlx runtime-tokio-rustls) and
# sqlx bundles libsqlite3-sys, so openssl-libs and sqlite-libs are not linked.
# procps-ng provides pgrep for the healthcheck; it is not present in ubi-minimal.
# hadolint ignore=DL3041
RUN microdnf update -y && \
    microdnf install -y --nodocs \
        ca-certificates \
        tzdata \
        procps-ng && \
    microdnf clean all && \
    rm -rf /var/cache/dnf

RUN useradd -m -u 1000 -s /bin/sh dicemaiden

COPY --from=builder --chmod=755 /app/target/release/dicemaiden-rs /usr/local/bin/dicemaiden-rs

# DATABASE_URL defaults to ./main.db, so the working directory must be writable.
WORKDIR /app
RUN chown dicemaiden:dicemaiden /app

USER dicemaiden

HEALTHCHECK --interval=30s --timeout=3s --start-period=60s --retries=3 \
    CMD pgrep -f dicemaiden-rs || exit 1

ENTRYPOINT ["/usr/local/bin/dicemaiden-rs"]
