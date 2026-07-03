FROM rust:1.91-bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace manifest
COPY Cargo.toml Cargo.lock ./

# Copy all package manifests for dependency caching
COPY konnektoren-core/Cargo.toml ./konnektoren-core/Cargo.toml
COPY konnektoren-platform/Cargo.toml ./konnektoren-platform/Cargo.toml
COPY konnektoren-tests/Cargo.toml ./konnektoren-tests/Cargo.toml
COPY konnektoren-tui/Cargo.toml ./konnektoren-tui/Cargo.toml
COPY konnektoren-page/Cargo.toml ./konnektoren-page/Cargo.toml

# Create dummy source files to build dependencies
RUN mkdir -p konnektoren-core/src && echo "pub fn dummy() {}" > konnektoren-core/src/lib.rs && \
    mkdir -p konnektoren-platform/src && echo "pub fn dummy() {}" > konnektoren-platform/src/lib.rs && \
    mkdir -p konnektoren-platform/tests && \
    echo "#[test]\nfn dummy() {}" > konnektoren-platform/tests/challenge_i18n.rs && \
    echo "#[test]\nfn dummy() {}" > konnektoren-platform/tests/manifest.rs && \
    echo "#[test]\nfn dummy() {}" > konnektoren-platform/tests/schema_export.rs && \
    mkdir -p konnektoren-tests/src && echo "pub fn dummy() {}" > konnektoren-tests/src/lib.rs && \
    mkdir -p konnektoren-tests/tests && echo "#[test]\nfn dummy() {}" > konnektoren-tests/tests/bdd_tests.rs && \
    mkdir -p konnektoren-tui/src && echo "pub fn dummy() {}" > konnektoren-tui/src/lib.rs && \
    mkdir -p konnektoren-tui/src/bin && echo "fn main() {}" > konnektoren-tui/src/bin/ssh-server.rs && \
    echo "fn main() {}" > konnektoren-tui/src/main.rs && \
    mkdir -p konnektoren-page/src && echo "fn main() {}" > konnektoren-page/src/main.rs

# Build dependencies (this layer will be cached)
RUN cargo check --release -p konnektoren-tui --bin konnektoren-tui-ssh --features ssh

# Remove dummy source files
RUN rm -rf konnektoren-core/src konnektoren-platform/src konnektoren-platform/tests konnektoren-tests/src konnektoren-tests/tests konnektoren-tui/src konnektoren-page/src

# Copy actual source code
COPY konnektoren-core ./konnektoren-core
COPY konnektoren-platform ./konnektoren-platform
COPY konnektoren-tests ./konnektoren-tests
COPY konnektoren-tui ./konnektoren-tui
COPY konnektoren-page ./konnektoren-page

# konnektoren-core embeds these via include_str!/include_bytes! at compile time
COPY assets ./assets

# Build the application
RUN cargo build --release -p konnektoren-tui --bin konnektoren-tui-ssh --features ssh

# Runtime stage
FROM debian:bookworm-slim AS runtime

WORKDIR /app

RUN apt-get update && apt-get install -y \
    ca-certificates \
    netcat-openbsd \
    iproute2 \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/konnektoren-tui-ssh /usr/local/bin/konnektoren-tui-ssh

RUN mkdir -p /app/data

# Create non-root user
RUN useradd -m -u 1000 -s /bin/bash konnektoren && \
    chown -R konnektoren:konnektoren /app

# Switch to non-root user
USER konnektoren

# Expose SSH port
EXPOSE 2222

ENV RUST_LOG=info

# Health check
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD sh -c 'ss -ltn sport = :2222 | grep -q LISTEN'

# Run the SSH server
CMD ["/usr/local/bin/konnektoren-tui-ssh"]
