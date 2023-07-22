#!/usr/bin/env bash
cd "$(dirname "${BASH_SOURCE[0]}")/.."

RED="\e[31m"
GREEN="\e[32m"
BLUE="\e[34m"
ENDCOLOR="\e[0m"

if [ -z "$1" ]
  then
    echo -e "${RED}No argument supplied${ENDCOLOR}"
    exit 1
fi

# Export env vars
export $(grep -v '^#' .env | xargs)
export DATABASE_URL="postgres://${DB_USER}:${DB_PASSWORD}@localhost:${DB_PORT}/${DB_NAME}"

echo ""
echo -e "${BLUE}Run Test \"$1\":${ENDCOLOR}
  DATABASE_URL: ${DATABASE_URL}
"

cargo test $1 -- --nocapture