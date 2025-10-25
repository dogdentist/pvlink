#!/bin/sh

docker compose -f compose.dev.yaml kill pvlink-backend
docker compose -f compose.dev.yaml start pvlink-backend
