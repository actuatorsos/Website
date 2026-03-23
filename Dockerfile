# ══════════════════════════════════════════════════════════════
# Actuators Platform — Production Docker Image
# SurrealDB v2 embedded + Actuators binary
# ══════════════════════════════════════════════════════════════

FROM rust:slim AS builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# ── Layer 1: Cache dependencies ──
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    echo "" > src/lib.rs
RUN cargo build --release --bin Actuators 2>/dev/null || true
RUN rm -rf src target/release/deps/Actuators* target/release/deps/libActuators* target/release/Actuators*

# ── Layer 2: Build application ──
COPY src ./src
COPY templates ./templates
RUN cargo build --release --bin Actuators

# ══════════════════════════════════════════════════════════════
# Production image with SurrealDB embedded
# ══════════════════════════════════════════════════════════════
FROM debian:bookworm-slim
WORKDIR /app

RUN apt-get update && \
    apt-get install -y --no-install-recommends libssl3 ca-certificates curl && \
    rm -rf /var/lib/apt/lists/*

# Install SurrealDB v2.4.0
RUN curl -sSf https://install.surrealdb.com | sh -s -- --version v2.4.0

# Binary
COPY --from=builder /app/target/release/Actuators /usr/local/bin/Actuators

# Static assets, templates, locales, schema
COPY static ./static
COPY locales ./locales
COPY templates ./templates
COPY src/db/schema.surql ./src/db/schema.surql
COPY data ./data

# Create directories
RUN mkdir -p static/uploads/videos static/uploads/courses /data/surreal

# Startup script
COPY entrypoint.sh /app/entrypoint.sh
RUN chmod +x /app/entrypoint.sh

# Environment defaults (override in DO App Platform)
ENV SERVER_HOST=0.0.0.0
ENV SERVER_PORT=8080
ENV RUST_LOG=info,actuators=debug

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=5s --start-period=15s --retries=3 \
    CMD curl -f http://localhost:8080/api/health || exit 1

CMD ["/app/entrypoint.sh"]
