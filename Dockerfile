FROM rust:slim AS builder
# Install nightly toolchain if required by the project
RUN rustup toolchain install nightly && rustup default nightly

WORKDIR /app

# Copy only manifests first for dependency caching
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    echo "" > src/lib.rs

# Build dependencies only (this layer is cached unless Cargo.toml/Cargo.lock change)
RUN cargo build --release || true
# Remove the dummy build artifacts so the real source gets compiled
RUN rm -rf src target/release/deps/Actuators* target/release/deps/libActuators* target/release/Actuators*

# Copy the actual source code
COPY src ./src

# Build the application
RUN cargo build --release

# We use debian slim for a smaller footprint in production
FROM debian:bookworm-slim
WORKDIR /app

# Install required libraries for Rust binary to work properly
RUN apt-get update && apt-get install -y libssl-dev ca-certificates libc-bin curl && rm -rf /var/lib/apt/lists/*

# Copy the build artifact from the builder stage
COPY --from=builder /app/target/release/dr_machine_web /usr/local/bin/dr_machine_web

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
CMD ["dr_machine_web"]
