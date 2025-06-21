FROM rust:1.87.0-alpine3.22 AS builder

RUN apk add --no-cache \
    musl-dev \
    pkgconfig \
    openssl-dev \
    openssl-libs-static

# Set up the working directory
WORKDIR /app

# Copy manifest files first for better caching
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (this layer will be cached)
RUN cargo build --release && rm -rf src

# Copy source code
COPY src ./src

# Build the application with musl target for static linking
RUN cargo build --release --target x86_64-unknown-linux-musl

# Strip the binary to reduce size further
RUN strip /app/target/x86_64-unknown-linux-musl/release/dicemaiden_rs

# Runtime stage - Ultra minimal distroless image (~2MB base)
FROM gcr.io/distroless/static

# Copy the statically linked binary from builder stage
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/dicemaiden-rs /dicemaiden-rs

# Run the application
ENTRYPOINT ["/dicemaiden-rs"]
