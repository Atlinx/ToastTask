#!/usr/bin/env bash
cd "$(dirname "${BASH_SOURCE[0]}")/.."

RED="\e[31m"
GREEN="\e[32m"
BLUE="\e[34m"
ENDCOLOR="\e[0m"

./scripts/init_db.sh

# Export env vars
export $(grep -v '^#' .env | xargs)
export DATABASE_URL="postgres://${DB_USER}:${DB_PASSWORD}@localhost:${DB_PORT}/${DB_NAME}"

echo ""
echo -e "${BLUE}Watching Tests:${ENDCOLOR}
  DATABASE_URL: ${DATABASE_URL}
"
cargo watch -x "test -- --nocapture"