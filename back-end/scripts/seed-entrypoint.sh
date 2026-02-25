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

echo "Validating draft order data..."
/app/seed-data draft-order validate --file /app/data/draft_order_2026.json
echo "Draft order validation complete."

echo "Seeding draft order..."
/app/seed-data draft-order load --file /app/data/draft_order_2026.json
echo "Draft order seeding complete."

# Rankings and scouting reports depend on both players and teams being loaded
RANKINGS_FILE="/app/data/rankings/rankings_2026.json"
if [ ! -f "$RANKINGS_FILE" ]; then
  # Try Tankathon as fallback
  if [ -f "/app/data/rankings/tankathon_2026.json" ]; then
    RANKINGS_FILE="/app/data/rankings/tankathon_2026.json"
    echo "Using Tankathon rankings as fallback."
  else
    echo "No scraped rankings found, generating template..."
    /app/prospect-rankings-scraper --template --output "$RANKINGS_FILE"
    echo "Template rankings generated."
  fi
fi

echo "Validating prospect rankings..."
/app/seed-data rankings validate --file "$RANKINGS_FILE"
echo "Rankings validation complete."

echo "Loading prospect rankings (with player discovery)..."
/app/seed-data rankings load --file "$RANKINGS_FILE"
echo "Rankings loading complete."

echo "Validating scouting report rankings..."
/app/seed-data scouting validate --file "$RANKINGS_FILE"
echo "Scouting report validation complete."

echo "Seeding scouting reports..."
/app/seed-data scouting load --file "$RANKINGS_FILE"
echo "Scouting report seeding complete."

echo "Validating Feldman Freaks data..."
/app/seed-data freaks validate --file /app/data/feldman_freaks_2026.json
echo "Feldman Freaks validation complete."

echo "Seeding Feldman Freaks..."
/app/seed-data freaks load --file /app/data/feldman_freaks_2026.json
echo "Feldman Freaks seeding complete."
