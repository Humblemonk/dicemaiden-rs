# Build arguments for version pinning
ARG RUST_VERSION=1.95.0
ARG UBI_VERSION=latest

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
# sqlx bundles libsqlite3-sys, so openssl-libs and sqlite-libs are not linked by the
# bot; `ldd` on the built binary shows only libgcc_s, libm and libc.
#
# sqlite and jq exist solely for the manual spot-check scripts in tools/. curl is already
# present in the base image as curl-minimal. None of these are used by the bot itself.
# hadolint ignore=DL3041
RUN microdnf update -y && \
    microdnf install -y --nodocs \
        ca-certificates \
        tzdata \
        sqlite \
        jq && \
    microdnf clean all && \
    rm -rf /var/cache/dnf && \
    sqlite3 --version && jq --version

RUN useradd -m -u 1000 -s /bin/sh dicemaiden

COPY --from=builder --chmod=755 /app/target/release/dicemaiden-rs /usr/local/bin/dicemaiden-rs

# Operator spot-check scripts, run by hand against a live deployment:
#   kubectl exec deploy/dicemaiden-rs -- topgg.sh --dry-run
# These live in /usr/local/bin rather than /app because the statistics database is a
# mounted volume in production, and the mount shadows anything placed under /app.
# dicemaiden-env.sh is sourced by both scripts via $(dirname "$0"), so it must sit
# alongside them.
COPY --chmod=755 tools/topgg.sh tools/quota.sh tools/dicemaiden-env.sh /usr/local/bin/

# DATABASE_URL defaults to ./main.db, so the working directory must be writable.
WORKDIR /app
RUN chown dicemaiden:dicemaiden /app

USER dicemaiden

ENTRYPOINT ["/usr/local/bin/dicemaiden-rs"]
