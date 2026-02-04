# Docker Deployment Guide for Backend API

This guide explains how to build and deploy the NFL Draft Simulator backend API using Docker.

## Multi-Stage Build

The Dockerfile uses a multi-stage build to create a minimal production image:

1. **Builder Stage**: Compiles the Rust application with all dependencies
2. **Runtime Stage**: Creates a minimal Debian image with only the compiled binary

Benefits:

- **Small Image Size**: ~100MB final image vs ~2GB with build tools
- **Security**: No build tools or source code in production image
- **Fast Deployment**: Smaller images deploy faster

## Prerequisites

- Docker 20.10+
- Docker Compose 2.0+ (for docker-compose deployment)
- SQLx offline metadata (`.sqlx/` directory)

### Generate SQLx Offline Metadata

SQLx requires metadata for compile-time query verification. Generate it before building:

```bash
# Install sqlx-cli if not already installed
cargo install sqlx-cli --no-default-features --features postgres

# Ensure PostgreSQL is running
docker compose up -d postgres

# Generate metadata
cargo sqlx prepare --workspace
```

This creates a `.sqlx/` directory with query metadata, allowing Docker to build without a database connection.

## Building the Docker Image

### Option 1: Using the Build Script

```bash
./build-docker.sh
```

The script:

- Checks for `.sqlx/` directory
- Builds the Docker image with tag `nfl-draft-api:latest`
- Provides next steps for running the container

### Option 2: Manual Docker Build

```bash
docker build -t nfl-draft-api:latest .
```

### Option 3: Using Docker Compose

```bash
# From repository root
docker compose build api
```

## Running the Container

### Option 1: Docker Run (Standalone)

```bash
docker run -d \
  --name nfl-draft-api \
  -p 8000:8000 \
  -e DATABASE_URL="postgresql://nfl_draft_user:nfl_draft_pass@host.docker.internal:5432/nfl_draft" \
  -e RUST_LOG=info \
  nfl-draft-api:latest
```

**Note**: Use `host.docker.internal` to connect to PostgreSQL running on the host machine.

### Option 2: Docker Compose (Recommended)

```bash
# From repository root

# Start PostgreSQL and API
docker compose up -d postgres api

# Check logs
docker compose logs -f api

# Stop services
docker compose down
```

### Option 3: Docker Compose with pgAdmin

```bash
# Start all services including pgAdmin
docker compose --profile tools up -d

# Access:
# - API: http://localhost:8000
# - pgAdmin: http://localhost:5050
```

## Environment Variables

Configure the container using environment variables:

| Variable       | Default                                                              | Description                    |
| -------------- | -------------------------------------------------------------------- | ------------------------------ |
| `DATABASE_URL` | `postgresql://nfl_draft_user:nfl_draft_pass@postgres:5432/nfl_draft` | PostgreSQL connection string   |
| `RUST_LOG`     | `info`                                                               | Log level (debug, info, error) |
| `SERVER_HOST`  | `0.0.0.0`                                                            | Server bind address            |
| `SERVER_PORT`  | `8000`                                                               | Server port                    |
| `API_PORT`     | `8000`                                                               | Host port mapping              |

### Using .env File

Create a `.env` file in the repository root:

```bash
# Database
POSTGRES_DB=nfl_draft
POSTGRES_USER=nfl_draft_user
POSTGRES_PASSWORD=nfl_draft_pass

# API
DATABASE_URL=postgresql://nfl_draft_user:nfl_draft_pass@postgres:5432/nfl_draft
RUST_LOG=debug
API_PORT=8000

# Optional: pgAdmin
PGADMIN_EMAIL=admin@nfldraft.local
PGADMIN_PASSWORD=admin
PGADMIN_PORT=5050
```

Then run:

```bash
docker compose up -d
```

## Health Checks

The container includes a health check that verifies the API is responding:

```bash
# Check container health
docker inspect --format='{{.State.Health.Status}}' nfl-draft-api

# Should return: healthy
```

The health check endpoint is `/health` and runs every 30 seconds.

## Database Migrations

### Option 1: Run Migrations Before Starting Container

```bash
# On host machine
cd back-end
sqlx migrate run
```

### Option 2: Run Migrations in Container

```bash
# Exec into container
docker compose exec api bash

# Run migrations (requires sqlx-cli in container)
# Note: Current Dockerfile doesn't include sqlx-cli in runtime
# This is intentional to keep image size small
```

### Option 3: Init Container (Recommended for Production)

Add an init container to docker-compose.yml:

```yaml
api-migrate:
  image: nfl-draft-api:latest
  command: /app/migrate # Custom migration script
  depends_on:
    postgres:
      condition: service_healthy
  networks:
    - nfl-draft-network
```

## Production Deployment

### Best Practices

1. **Use Specific Tags**:

   ```bash
   docker build -t nfl-draft-api:1.0.0 .
   docker tag nfl-draft-api:1.0.0 nfl-draft-api:latest
   ```

2. **Use Multi-Platform Builds** (for ARM64 support):

   ```bash
   docker buildx build \
     --platform linux/amd64,linux/arm64 \
     -t nfl-draft-api:1.0.0 \
     --push .
   ```

3. **Resource Limits** (in docker-compose.yml):

   ```yaml
   api:
     deploy:
       resources:
         limits:
           cpus: "1.0"
           memory: 512M
         reservations:
           cpus: "0.5"
           memory: 256M
   ```

4. **Security**:
   - Container runs as non-root user (UID 1000)
   - No unnecessary tools in runtime image
   - Use secrets for sensitive environment variables

### Container Registry

Push to a container registry for deployment:

```bash
# Tag for registry
docker tag nfl-draft-api:latest your-registry.com/nfl-draft-api:1.0.0

# Push to registry
docker push your-registry.com/nfl-draft-api:1.0.0
```

## Troubleshooting

### Build Fails: SQLx Query Verification

**Error**: `error: no database URL provided`

**Solution**: Generate SQLx offline metadata:

```bash
cargo sqlx prepare --workspace
```

### Container Can't Connect to Database

**Error**: `connection refused` or `could not connect to server`

**Solution**: Ensure PostgreSQL is running and accessible:

```bash
# Check PostgreSQL is running
docker compose ps postgres

# Check network connectivity
docker compose exec api ping postgres

# Verify DATABASE_URL
docker compose exec api env | grep DATABASE_URL
```

### Health Check Failing

**Error**: Container stuck in `unhealthy` state

**Solution**:

```bash
# Check logs
docker compose logs api

# Verify health endpoint manually
docker compose exec api curl http://localhost:8000/health
```

### Large Image Size

**Problem**: Image is larger than expected

**Solution**:

- Ensure multi-stage build is working correctly
- Check that builder artifacts aren't copied to runtime stage
- Verify `.dockerignore` is excluding unnecessary files

```bash
# Check image size
docker images nfl-draft-api

# Should be ~100-200MB for runtime image
```

## Image Optimization

Current optimizations:

- ✅ Multi-stage build (builder + runtime)
- ✅ Debian slim base image
- ✅ Dependency caching layer
- ✅ .dockerignore to reduce context size
- ✅ Non-root user
- ✅ Minimal runtime dependencies

Future optimizations:

- Use `distroless` base image (even smaller, ~50MB total)
- Enable link-time optimization (LTO) in release build
- Use `musl` for fully static binary (Alpine base)

## Useful Commands

```bash
# View image layers and sizes
docker history nfl-draft-api:latest

# Inspect image
docker inspect nfl-draft-api:latest

# Run with shell for debugging
docker run -it --entrypoint /bin/bash nfl-draft-api:latest

# View container logs
docker compose logs -f api

# Restart container
docker compose restart api

# Rebuild and restart
docker compose up -d --build api

# Remove all containers and volumes (destructive)
docker compose down -v
```

## CI/CD Integration

Example GitHub Actions workflow:

```yaml
name: Build and Push Docker Image

on:
  push:
    branches: [main]
    tags: ["v*"]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v2

      - name: Login to Container Registry
        uses: docker/login-action@v2
        with:
          registry: ghcr.io
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - name: Generate SQLx Metadata
        run: |
          cd back-end
          cargo install sqlx-cli --no-default-features --features postgres
          cargo sqlx prepare --workspace

      - name: Build and Push
        uses: docker/build-push-action@v4
        with:
          context: ./back-end
          push: true
          tags: |
            ghcr.io/${{ github.repository }}/api:latest
            ghcr.io/${{ github.repository }}/api:${{ github.sha }}
          cache-from: type=gha
          cache-to: type=gha,mode=max
```

## References

- [Docker Multi-Stage Builds](https://docs.docker.com/build/building/multi-stage/)
- [Rust Docker Best Practices](https://docs.docker.com/language/rust/)
- [SQLx Offline Mode](https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md#enable-building-in-offline-mode-with-query)
