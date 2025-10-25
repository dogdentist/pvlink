#!/bin/sh

mkdir -p /tmp/pvlink/analytic-cargo
docker compose -f compose.dev.yaml kill pvlink-analytic
docker compose -f compose.dev.yaml start pvlink-analytic
