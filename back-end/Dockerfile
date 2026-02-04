# Multi-stage Dockerfile for NFL Draft Simulator Backend
# Stage 1: Build the application
FROM rust:1.83-bookworm AS builder

# Install system dependencies required for building
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create a new empty shell project
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY crates/api/Cargo.toml ./crates/api/
COPY crates/domain/Cargo.toml ./crates/domain/
COPY crates/db/Cargo.toml ./crates/db/
COPY crates/websocket/Cargo.toml ./crates/websocket/

# Build dependencies only (this layer will be cached)
# Create dummy source files to compile dependencies
RUN mkdir -p crates/api/src crates/domain/src crates/db/src crates/websocket/src && \
    echo "fn main() {}" > crates/api/src/main.rs && \
    echo "pub fn dummy() {}" > crates/domain/src/lib.rs && \
    echo "pub fn dummy() {}" > crates/db/src/lib.rs && \
    echo "pub fn dummy() {}" > crates/websocket/src/lib.rs && \
    cargo build --release && \
    rm -rf crates/*/src

# Copy the actual source code
COPY crates ./crates

# Note: .sqlx directory is not needed for Docker builds
# SQLx queries are validated at compile time using DATABASE_URL

# Build the actual application
# Touch the files to ensure cargo rebuilds them
RUN touch crates/api/src/main.rs && \
    touch crates/domain/src/lib.rs && \
    touch crates/db/src/lib.rs && \
    touch crates/websocket/src/lib.rs && \
    cargo build --release -p api

# Stage 2: Create the runtime image
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user to run the application
RUN useradd -m -u 1000 appuser

WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/api /app/api

# Copy migrations directory (needed for runtime if using embedded migrations)
COPY migrations ./migrations

# Change ownership to the non-root user
RUN chown -R appuser:appuser /app

# Switch to non-root user
USER appuser

# Expose the application port
EXPOSE 8000

# Set environment variables
ENV RUST_LOG=info
ENV DATABASE_URL=postgresql://nfl_draft_user:nfl_draft_pass@postgres:5432/nfl_draft

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8000/health || exit 1

# Run the binary
CMD ["/app/api"]
