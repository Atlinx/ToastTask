#!/usr/bin/env bash
cd "$(dirname "${BASH_SOURCE[0]}")/.."

RED="\e[31m"
GREEN="\e[32m"
ENDCOLOR="\e[0m"

echo -e "${RED}Stopping PostgresDB:${ENDCOLOR}"
docker stop postgres | sed 's/^/  /' || true && docker rm postgres -v | sed 's/^/  /' || true