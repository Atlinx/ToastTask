#!/usr/bin/env bash
cd "$(dirname "${BASH_SOURCE[0]}")/.."

RED="\e[31m"
GREEN="\e[32m"
ENDCOLOR="\e[0m"

echo -e "${GREEN}Initialize PostgresDB:${ENDCOLOR}"
echo ""
echo -e "${GREEN}Loading .ENV:${ENDCOLOR}"
# Load dotenv variables
# Show env vars
grep -v '^#' .env | sed 's/^/  /'
# Export env vars
export $(grep -v '^#' .env | xargs)
export DATABASE_URL="postgres://${DB_USER}:${DB_PASSWORD}@localhost:${DB_PORT}/${DB_NAME}"
echo ""

if ! [ -x "$(command -v psql)" ]; then
  echo -e "${RED}Error: `psql` is not installed.${ENDCOLOR}"
  exit 1
fi

if ! [ -x "$(command -v sqlx)" ]; then
  echo "${RED}Error: `sqlx` is not installed.${ENDCOLOR}

Use:
cargo install --version=0.5.7 sqlx-cli --no-default-features --features postgres
to install it."
  exit 1
fi

./scripts/stop_db.sh
echo -e "${GREEN}Starting Postgres Container:${ENDCOLOR}"
docker run --name postgres \
-e POSTGRES_USER=${DB_USER} \
-e POSTGRES_PASSWORD=${DB_PASSWORD} \
-e POSTGRES_DB=${DB_NAME} \
-p "${DB_PORT}":5432 \
-d postgres postgres -N 1000 | sed 's/^/  /'

echo ""
echo -e "${RED}Waiting...${ENDCOLOR}"
export PGPASSWORD="${DB_PASSWORD}"
until psql -h "localhost" -U "${DB_USER}" -p "${DB_PORT}" -d "postgres" -c '\q' > /dev/null 2>&1; do
  echo "  Postgres is still unavailable - sleeping"
  sleep 1
done

echo ""
echo -e "${GREEN}Postgres is up and running on port ${DB_PORT}${ENDCOLOR}"

sqlx migrate run | sed 's/^/  /'