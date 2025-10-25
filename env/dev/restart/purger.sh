#!/bin/sh

mkdir -p /tmp/pvlink/purger-cargo
docker compose -f compose.dev.yaml kill pvlink-purger
docker compose -f compose.dev.yaml start pvlink-purger
