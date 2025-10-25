#!/bin/sh

mkdir -p /tmp/pvlink/shortener-cargo
mkdir -p /tmp/pvlink/analytic-cargo
mkdir -p /tmp/pvlink/purger-cargo

docker compose -f compose.dev.yaml up --build -d
