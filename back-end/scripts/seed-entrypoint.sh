#!/bin/bash
set -e

echo "Running database migrations..."
sqlx migrate run --source /app/migrations
echo "Migrations complete."

echo "Seeding team data..."
/app/seed-data teams load --file /app/data/teams_nfl.json
echo "Team seeding complete."

echo "Seeding player data..."
/app/seed-data players load --file /app/data/players_2026.json
echo "Player seeding complete."

echo "Seeding team positional needs..."
/app/seed-data needs load --file /app/data/team_needs_2026.json
echo "Team needs seeding complete."

echo "Seeding team season records..."
/app/seed-data seasons load --file /app/data/team_seasons_2025.json
echo "Team seasons seeding complete."
