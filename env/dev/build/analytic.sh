#!/bin/sh

mkdir -p /tmp/pvlink/analytic-cargo
docker compose -f compose.dev.yaml build --no-cache pvlink-analytic
