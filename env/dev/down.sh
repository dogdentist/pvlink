#!/bin/sh

docker compose -f compose.dev.yaml kill
docker compose -f compose.dev.yaml down
