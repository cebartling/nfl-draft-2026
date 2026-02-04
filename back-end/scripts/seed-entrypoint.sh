#!/bin/bash
set -e

echo "Running database migrations..."
sqlx migrate run --source /app/migrations
echo "Migrations complete."

echo "Seeding player data..."
/app/seed-players load --file /app/data/players_2026.json
echo "Seeding complete."
