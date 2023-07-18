#!/usr/bin/env bash
cd "$(dirname "${BASH_SOURCE[0]}")/.."

RED="\e[31m"
GREEN="\e[32m"
BLUE="\e[34m"
ENDCOLOR="\e[0m"

echo -e "${GREEN}Loading .ENV:${ENDCOLOR}"
# Load dotenv variables
# Show env vars
grep -v '^#' .env | sed 's/^/  /'
# Export env vars
export $(grep -v '^#' .env | xargs)
export DATABASE_URL="postgres://${DB_USER}:${DB_PASSWORD}@localhost:${DB_PORT}/${DB_NAME}"
echo ""

echo -e "${GREEN}Prepare PostgresDB:${ENDCOLOR}"
cargo sqlx prepare | sed 's/^/  /'