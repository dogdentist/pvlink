#!/bin/sh

docker compose -f compose.dev.yaml kill pvlink-db
docker compose -f compose.dev.yaml down pvlink-db
docker compose -f compose.dev.yaml start pvlink-db
