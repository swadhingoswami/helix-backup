# =============================================================================
# HELIX - Multi-stage Docker build
# =============================================================================

# Stage 1: Builder
FROM rust:1.76-slim-bookworm AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libsqlite3-dev \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release 2>/dev/null || true

COPY src/ src/
RUN cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    libsqlite3-0 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -r -s /bin/false helix

COPY --from=builder /app/target/release/helix /usr/local/bin/helix

RUN mkdir -p /var/helix/backups /etc/helix /var/log/helix && \
    chown -R helix:helix /var/helix /etc/helix /var/log/helix

USER helix

ENTRYPOINT ["/usr/local/bin/helix"]
CMD ["--help"]
