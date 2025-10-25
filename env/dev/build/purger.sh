#!/bin/sh

mkdir -p /tmp/pvlink/purger-cargo
docker compose -f compose.dev.yaml build --no-cache pvlink-purger
