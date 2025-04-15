# Build stage
FROM rust:1.86-slim AS builder

WORKDIR /usr/src/otp

# Copy over the manifest
COPY Cargo.toml ./

# Create a dummy main.rs to build dependencies
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy the actual source code
COPY . .

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install necessary runtime dependencies and debugging tools
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates strace procps && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from the builder stage
COPY --from=builder /usr/src/otp/target/release/otp /app/otp

# Create a non-root user to run the application
RUN useradd -m otp-user && \
    chown -R otp-user:otp-user /app

USER otp-user

# Expose the port the server listens on
EXPOSE 8080

# Set environment variables
ENV SERVER_HOST=0.0.0.0
ENV SERVER_PORT=8080
ENV LOG_LEVEL=info
ENV OTP_LENGTH=6
ENV OTP_EXPIRY_SECONDS=30
ENV STORAGE_CLEANUP_INTERVAL=60
ENV STORAGE_TYPE=redis
ENV REDIS_URL=redis://redis:6379

# Run the application with proper output
CMD ["./otp"]
