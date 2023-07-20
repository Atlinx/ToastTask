#!/usr/bin/env bash
cd "$(dirname "${BASH_SOURCE[0]}")/.."

RED="\e[31m"
GREEN="\e[32m"
BLUE="\e[34m"
ENDCOLOR="\e[0m"

DEFAULT_RUN_COUNT=1
RUN_COUNT="${1:-$DEFAULT_RUN_COUNT}"

# Export env vars
export $(grep -v '^#' .env | xargs)
export DATABASE_URL="postgres://${DB_USER}:${DB_PASSWORD}@localhost:${DB_PORT}/${DB_NAME}"

if ! [[ -v SKIP_INIT_DB ]]; then
  ./scripts/init_db.sh
fi

echo ""
echo -e "${BLUE}Run Tests ${RUN_COUNT} time(s):${ENDCOLOR}
  DATABASE_URL: ${DATABASE_URL}
"

i=1
while [[ $i -le $RUN_COUNT ]]
do 
echo -e "${BLUE}Run #${i}:${ENDCOLOR}
"
  cargo test
  i=$(($i+1))
done
