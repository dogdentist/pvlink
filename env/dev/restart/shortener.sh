#!/bin/sh

mkdir -p /tmp/pvlink/shortener-cargo
docker compose -f compose.dev.yaml kill pvlink-shortener
docker compose -f compose.dev.yaml start pvlink-shortener
