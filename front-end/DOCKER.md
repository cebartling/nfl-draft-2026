# Frontend Docker Documentation

This document covers the containerization of the NFL Draft Frontend application using Docker and nginx.

## Overview

The frontend is containerized using a **multi-stage Docker build** that produces a lightweight production image:

- **Stage 1 (Builder)**: Builds the SvelteKit application with Node.js 20
- **Stage 2 (Runtime)**: Serves static files with nginx Alpine (~25MB)

**Key Features:**

- Static file serving optimized for performance
- Reverse proxy for backend API (`/api/*`)
- WebSocket proxy for real-time updates (`/ws`)
- Gzip compression for assets
- Aggressive caching for static files
- Non-root user for security
- Health check endpoint

## Architecture

```
┌─────────────────────────────────────────────┐
│          Frontend Container (nginx)         │
│                                             │
│  ┌─────────────────────────────────────┐  │
│  │     Static Files (/usr/share/nginx)  │  │
│  │  - HTML, CSS, JS from SvelteKit      │  │
│  └─────────────────────────────────────┘  │
│                                             │
│  ┌─────────────────────────────────────┐  │
│  │      Reverse Proxy (nginx)           │  │
│  │  - /api/* → api:8000                 │  │
│  │  - /ws    → api:8000 (WebSocket)     │  │
│  └─────────────────────────────────────┘  │
└─────────────────────────────────────────────┘
                    ▼
         ┌─────────────────────┐
         │  Backend Container   │
         │  (Rust API: 8000)    │
         └─────────────────────┘
```

## Building the Image

### Using the Build Script (Recommended)

```bash
cd front-end

# Build with defaults
./build-docker.sh

# Build with custom name/tag
DOCKER_IMAGE_NAME=my-frontend DOCKER_IMAGE_TAG=v1.0.0 ./build-docker.sh

# Build for different platform (e.g., ARM)
DOCKER_PLATFORM=linux/arm64 ./build-docker.sh
```

### Manual Build

```bash
cd front-end

docker build -t nfl-draft-frontend:latest .
```

### Build Arguments

The build script supports the following environment variables:

| Variable            | Default              | Description       |
| ------------------- | -------------------- | ----------------- |
| `DOCKER_IMAGE_NAME` | `nfl-draft-frontend` | Docker image name |
| `DOCKER_IMAGE_TAG`  | `latest`             | Docker image tag  |
| `DOCKER_PLATFORM`   | `linux/amd64`        | Target platform   |

## Running the Container

### Standalone (Frontend Only)

**Note:** Running standalone requires backend API to be accessible at `api:8000`. Use docker compose for full stack.

```bash
docker run -p 3000:8080 nfl-draft-frontend:latest
```

Access at: http://localhost:3000

### With Docker Compose (Recommended)

```bash
# From repository root
docker compose up frontend

# Or start full stack
docker compose up -d
```

This automatically:

- Starts PostgreSQL
- Starts backend API
- Starts frontend with proper networking

### Custom Port Mapping

```bash
# Map to different host port
docker run -p 8080:8080 nfl-draft-frontend:latest

# Access at http://localhost:8080
```

## Configuration

### Nginx Configuration

The nginx configuration (`nginx.conf`) handles:

1. **Static File Serving**
   - Root: `/usr/share/nginx/html` (SvelteKit build output)
   - Fallback: `index.html` (SPA routing)
   - Cache: 1 year for static assets (js, css, images)

2. **API Reverse Proxy** (`/api/*`)
   - Forwards to: `http://api:8000/api/`
   - Timeout: 60 seconds
   - Keeps original headers (X-Forwarded-For, etc.)

3. **WebSocket Proxy** (`/ws`)
   - Forwards to: `http://api:8000/ws`
   - Timeout: 7 days (persistent connections)
   - Upgrade headers for WebSocket protocol

4. **Compression**
   - Gzip enabled for text/JSON/JS/CSS
   - Minimum size: 1024 bytes
   - Level: 6 (balanced)

### Environment Variables

The container itself doesn't require environment variables, but docker-compose passes:

| Variable        | Default | Description         |
| --------------- | ------- | ------------------- |
| `FRONTEND_PORT` | `3000`  | Host port to expose |

Backend connection is hardcoded to `api:8000` in nginx config (docker network).

## Health Checks

### Docker Health Check

Built into the image:

```dockerfile
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:8080/health || exit 1
```

### Manual Health Check

```bash
# Check container health status
docker ps

# Test health endpoint
curl http://localhost:3000/health
# Expected: "healthy"
```

## Security

### Non-Root User

The container runs as user `appuser` (UID 1000, GID 1000):

```dockerfile
USER appuser
```

This prevents privilege escalation and follows security best practices.

### Port Configuration

- **Container port:** 8080 (non-privileged)
- **Host port:** 3000 (configurable via docker-compose)

Nginx runs on port 8080 (not 80) to allow rootless operation.

### Security Headers

Nginx adds security headers:

- `X-Frame-Options: SAMEORIGIN` (prevent clickjacking)
- `X-Content-Type-Options: nosniff` (prevent MIME sniffing)
- `X-XSS-Protection: 1; mode=block` (XSS protection)

## Troubleshooting

### Container Won't Start

**Check logs:**

```bash
docker logs nfl-draft-frontend
```

**Common issues:**

1. Port 3000 already in use
   - Solution: Use different port: `docker run -p 8080:8080 ...`

2. Backend not accessible
   - Solution: Ensure backend is running and on same network

### Static Files Not Loading

**Verify build output:**

```bash
docker run --rm nfl-draft-frontend:latest ls -la /usr/share/nginx/html
```

Should show:

- `index.html`
- `_app/` directory with JS/CSS
- Other static assets

**Check nginx config:**

```bash
docker run --rm nfl-draft-frontend:latest cat /etc/nginx/nginx.conf
```

### API Requests Failing

**Test API proxy:**

```bash
# From host machine
curl http://localhost:3000/api/v1/teams

# Should return list of teams (may be empty array initially)
```

**Check backend connectivity:**

```bash
# From inside container
docker exec -it nfl-draft-frontend wget -O- http://api:8000/health
```

### WebSocket Connection Issues

**Test WebSocket endpoint:**

```bash
# Use websocat or similar tool
websocat ws://localhost:3000/ws
```

**Check nginx logs:**

```bash
docker logs nfl-draft-frontend 2>&1 | grep upgrade
```

Should show upgrade requests for WebSocket connections.

## Performance Optimization

### Caching Strategy

**Static Assets (1 year cache):**

- `.js`, `.css`, `.png`, `.jpg`, `.woff`, `.svg`, etc.
- Header: `Cache-Control: public, immutable`
- Nginx serves from memory when possible

**HTML Files (no cache):**

- `index.html` and SPA routes
- Header: `Cache-Control: no-cache, no-store, must-revalidate`
- Always fetches latest version

### Gzip Compression

Enabled for:

- Text files (HTML, CSS, JS, JSON)
- Fonts (TTF, WOFF)
- SVG images

**Compression level:** 6 (balanced speed/size)

### Image Size

**Target size:** ~25-50MB

**Multi-stage build benefits:**

- Node.js and build tools stay in builder stage
- Runtime image only has nginx + static files
- ~6x smaller than including Node.js runtime

**Check image size:**

```bash
docker images nfl-draft-frontend:latest
```

## Production Deployment

### Using Docker Compose

**Recommended for production:**

```yaml
# docker-compose.yml (repository root)
services:
  frontend:
    build:
      context: ./front-end
      dockerfile: Dockerfile
    restart: unless-stopped
    ports:
      - '3000:8080'
    depends_on:
      api:
        condition: service_healthy
    networks:
      - nfl-draft-network
```

**Deploy:**

```bash
docker compose up -d frontend
```

### Environment-Specific Builds

**Development:**

```bash
DOCKER_IMAGE_TAG=dev ./build-docker.sh
```

**Staging:**

```bash
DOCKER_IMAGE_TAG=staging ./build-docker.sh
```

**Production:**

```bash
DOCKER_IMAGE_TAG=prod ./build-docker.sh
```

### Kubernetes Deployment

**Deployment example:**

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: nfl-draft-frontend
spec:
  replicas: 3
  selector:
    matchLabels:
      app: nfl-draft-frontend
  template:
    metadata:
      labels:
        app: nfl-draft-frontend
    spec:
      containers:
        - name: frontend
          image: nfl-draft-frontend:latest
          ports:
            - containerPort: 8080
          livenessProbe:
            httpGet:
              path: /health
              port: 8080
            initialDelaySeconds: 5
            periodSeconds: 10
          readinessProbe:
            httpGet:
              path: /health
              port: 8080
            initialDelaySeconds: 5
            periodSeconds: 10
          resources:
            requests:
              memory: '64Mi'
              cpu: '100m'
            limits:
              memory: '128Mi'
              cpu: '200m'
```

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Build Frontend Docker Image

on:
  push:
    branches: [main]
    paths:
      - 'front-end/**'

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Build Docker image
        run: |
          cd front-end
          DOCKER_IMAGE_TAG=${{ github.sha }} ./build-docker.sh

      - name: Test image
        run: |
          docker run -d -p 8080:8080 --name test nfl-draft-frontend:${{ github.sha }}
          sleep 5
          curl -f http://localhost:8080/health || exit 1
          docker stop test

      - name: Push to registry
        if: github.ref == 'refs/heads/main'
        run: |
          echo "${{ secrets.DOCKER_PASSWORD }}" | docker login -u "${{ secrets.DOCKER_USERNAME }}" --password-stdin
          docker tag nfl-draft-frontend:${{ github.sha }} myregistry/nfl-draft-frontend:latest
          docker push myregistry/nfl-draft-frontend:latest
```

## Monitoring

### Nginx Access Logs

```bash
# Follow access logs
docker logs -f nfl-draft-frontend

# Filter for errors
docker logs nfl-draft-frontend 2>&1 | grep error
```

### Metrics Collection

**Prometheus nginx-exporter:**

```bash
# Add to docker-compose.yml
services:
  nginx-exporter:
    image: nginx/nginx-prometheus-exporter:latest
    command:
      - -nginx.scrape-uri=http://frontend:8080/nginx_status
    ports:
      - "9113:9113"
```

## Additional Resources

- [Nginx Official Documentation](https://nginx.org/en/docs/)
- [SvelteKit Static Adapter](https://kit.svelte.dev/docs/adapter-static)
- [Docker Multi-Stage Builds](https://docs.docker.com/build/building/multi-stage/)
- [Docker Security Best Practices](https://docs.docker.com/develop/security-best-practices/)

## Support

For issues or questions:

1. Check this documentation first
2. Review nginx logs: `docker logs nfl-draft-frontend`
3. Verify backend connectivity
4. Check GitHub issues
