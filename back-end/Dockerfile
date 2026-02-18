# Multi-stage Dockerfile for NFL Draft Simulator Backend
# Stage 1: Build the application
FROM rust:1.85-bookworm AS builder

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
COPY crates/seed-data/Cargo.toml ./crates/seed-data/
COPY crates/draft-order-scraper/Cargo.toml ./crates/draft-order-scraper/
COPY crates/prospect-rankings-scraper/Cargo.toml ./crates/prospect-rankings-scraper/
COPY crates/combine-data-scraper/Cargo.toml ./crates/combine-data-scraper/

# Build dependencies only (this layer will be cached)
# Create dummy source files to compile dependencies
RUN mkdir -p crates/api/src crates/domain/src crates/db/src crates/websocket/src crates/seed-data/src crates/draft-order-scraper/src crates/prospect-rankings-scraper/src crates/combine-data-scraper/src && \
    echo "fn main() {}" > crates/api/src/main.rs && \
    echo "pub fn dummy() {}" > crates/domain/src/lib.rs && \
    echo "pub fn dummy() {}" > crates/db/src/lib.rs && \
    echo "pub fn dummy() {}" > crates/websocket/src/lib.rs && \
    echo "pub fn dummy() {}" > crates/seed-data/src/lib.rs && \
    echo "fn main() {}" > crates/seed-data/src/main.rs && \
    echo "fn main() {}" > crates/draft-order-scraper/src/main.rs && \
    echo "fn main() {}" > crates/prospect-rankings-scraper/src/main.rs && \
    echo "fn main() {}" > crates/combine-data-scraper/src/main.rs && \
    cargo build --release && \
    rm -rf crates/*/src

# Copy the actual source code
COPY crates ./crates

# Copy embedded data files (used by include_str! in api crate)
COPY data ./data

# Copy SQLx offline query cache for compile-time verification
COPY .sqlx ./.sqlx

# Build the actual application
# Touch the files to ensure cargo rebuilds them
RUN touch crates/api/src/main.rs && \
    touch crates/domain/src/lib.rs && \
    touch crates/db/src/lib.rs && \
    touch crates/websocket/src/lib.rs && \
    touch crates/seed-data/src/lib.rs && \
    touch crates/seed-data/src/main.rs && \
    touch crates/draft-order-scraper/src/main.rs && \
    touch crates/prospect-rankings-scraper/src/main.rs && \
    touch crates/combine-data-scraper/src/main.rs && \
    cargo build --release -p api

# Stage 2: Create the runtime image
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user to run the application
RUN useradd -m -u 1000 appuser

WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/api /app/api

# Copy migrations directory (needed for runtime if using embedded migrations)
COPY migrations ./migrations

# Copy data files (draft order JSON, etc.)
COPY --from=builder /app/data ./data

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
