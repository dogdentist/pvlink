#!/bin/sh

docker compose -f compose.dev.yaml kill pvlink-frontend
docker compose -f compose.dev.yaml start pvlink-frontend
