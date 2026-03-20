FROM rust:slim AS builder

WORKDIR /app

# Copy only manifests first for dependency caching
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    echo "" > src/lib.rs

# Build dependencies only (this layer is cached unless Cargo.toml/Cargo.lock change)
RUN cargo build --release --bin Actuators 2>/dev/null || true
# Remove the dummy build artifacts so the real source gets compiled
RUN rm -rf src target/release/deps/Actuators* target/release/deps/libActuators* target/release/Actuators*

# Copy the actual source code
COPY src ./src
COPY templates ./templates

# Build the application
RUN cargo build --release --bin Actuators

# Production image — debian slim for a smaller footprint
FROM debian:bookworm-slim
WORKDIR /app

# Install required runtime libraries
RUN apt-get update && apt-get install -y libssl-dev ca-certificates curl && rm -rf /var/lib/apt/lists/*

# Copy the build artifact from the builder stage
COPY --from=builder /app/target/release/Actuators /usr/local/bin/Actuators

# Copy necessary static assets
COPY static ./static
COPY locales ./locales
COPY templates ./templates

# Copy database schema
COPY src/db/schema.surql ./src/db/schema.surql

# Set environment variables
ENV SERVER_HOST=0.0.0.0
ENV SERVER_PORT=3000
ENV RUST_LOG=info

# Expose the application port
EXPOSE 3000

# Run the application
CMD ["Actuators"]
