#!/bin/sh

mkdir -p /tmp/pvlink/shortener-cargo
docker compose -f compose.dev.yaml build --no-cache pvlink-shortener
