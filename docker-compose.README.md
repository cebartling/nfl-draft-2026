# Docker Compose Setup

## Prerequisites

- Docker Desktop installed and running
- Docker Compose v2+ (included with Docker Desktop)

## Quick Start

1. **Copy environment variables:**
   ```bash
   cp .env.example .env
   ```

2. **Start PostgreSQL:**
   ```bash
   docker compose up -d postgres
   ```

3. **Verify PostgreSQL is running:**
   ```bash
   docker compose ps
   docker compose logs postgres
   ```

4. **Connect to database:**
   ```bash
   # Via psql in container
   docker compose exec postgres psql -U nfl_draft_user -d nfl_draft

   # Or use connection string from .env
   # postgresql://nfl_draft_user:nfl_draft_pass@localhost:5432/nfl_draft
   ```

## Services

### PostgreSQL 18

- **Port:** 5432 (configurable via `POSTGRES_PORT` in `.env`)
- **Database:** `nfl_draft`
- **User:** `nfl_draft_user`
- **Password:** `nfl_draft_pass`

Data is persisted in a Docker volume named `postgres_data`.

### pgAdmin (Optional)

A web-based PostgreSQL administration tool.

**Start with:**
```bash
docker compose --profile tools up -d
```

**Access:**
- URL: http://localhost:5050
- Email: `admin@nfldraft.local`
- Password: `admin`

**Add PostgreSQL Server in pgAdmin:**
1. Right-click "Servers" → "Register" → "Server"
2. General tab: Name = "NFL Draft Local"
3. Connection tab:
   - Host: `postgres` (Docker network name)
   - Port: `5432`
   - Database: `nfl_draft`
   - Username: `nfl_draft_user`
   - Password: `nfl_draft_pass`
   - Save password: Yes

## Common Commands

```bash
# Start all services
docker compose up -d

# Start with tools (pgAdmin)
docker compose --profile tools up -d

# View logs
docker compose logs -f postgres

# Stop services
docker compose down

# Stop and remove all data (destructive!)
docker compose down -v

# Restart PostgreSQL
docker compose restart postgres

# Execute SQL file
docker compose exec -T postgres psql -U nfl_draft_user -d nfl_draft < script.sql
```

## Troubleshooting

### Port 5432 already in use

If you have PostgreSQL installed locally, either:
1. Stop local PostgreSQL: `brew services stop postgresql` (macOS)
2. Change port in `.env`: `POSTGRES_PORT=5433`

### Database connection refused

Check if PostgreSQL is healthy:
```bash
docker compose ps
docker compose logs postgres
```

Wait for the healthcheck to pass (can take 10-30 seconds on first start).

### Reset database

```bash
# Stop and remove volumes
docker compose down -v

# Start fresh
docker compose up -d postgres
```

## Database Migrations

After starting PostgreSQL, run migrations with sqlx:

```bash
# Install sqlx-cli (first time only)
cargo install sqlx-cli --no-default-features --features postgres

# Run migrations
sqlx migrate run

# Create new migration
sqlx migrate add create_table_name
```
