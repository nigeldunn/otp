# ---- Build Stage ----
FROM rust:1.88 as builder

# Set working directory
WORKDIR /usr/src/otp_server

# Install build dependencies (if any, e.g., openssl)
# RUN apt-get update && apt-get install -y --no-install-recommends openssl libssl-dev pkg-config && rm -rf /var/lib/apt/lists/*
# Note: Uncomment and adjust the above line if your project has C dependencies like OpenSSL

# Copy manifests
COPY Cargo.toml Cargo.lock* ./
# Build dependencies only to leverage Docker cache
# Create a dummy main.rs to allow building dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy source code
COPY src ./src

# Build the application
# Clean previous dummy build artifacts
RUN rm -f target/release/deps/otp*
RUN cargo build --release

# ---- Runtime Stage ----
FROM debian:12-slim

# Set working directory
WORKDIR /app

# Create a non-root user and group
RUN groupadd -r appuser && useradd -r -g appuser appuser

# Install runtime dependencies (if any, e.g., ca-certificates for HTTPS)
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the build stage
COPY --from=builder /usr/src/otp_server/target/release/otp ./otp

# Copy the start script
COPY start-server.sh ./start-server.sh
RUN chmod +x ./start-server.sh

# Ensure the binary is executable by the appuser
RUN chown appuser:appuser ./otp ./start-server.sh

# Switch to the non-root user
USER appuser

# Expose the application port
EXPOSE 8080

# Set the entrypoint to the start script
CMD ["./start-server.sh"]
