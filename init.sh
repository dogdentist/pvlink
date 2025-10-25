#!/bin/bash

if [ ! -f "env/init.conf" ]; then
    echo "missing env/init.conf, configure env/init.conf.template and save it as env/init.conf"
    exit 1
fi

echo "loading env/init.conf"
source env/init.conf

echo "configuring traefik"

convert_to_yaml_array() {
    local ips="$1"
    echo "$ips" | sed 's/,/","/g' | sed 's/^/["/' | sed 's/$/"]/'
}

TRAEFIK_IPS=$(convert_to_yaml_array "$TRAEFIK_WHITELIST_IP_RANGE_LIST")
ADMIN_IPS=$(convert_to_yaml_array "$ADMIN_DASHBOARD_WHITELIST_IP_RANGE_LIST")
PROXY_WHITELIST_IPS=$(convert_to_yaml_array "$TRAEFIK_TRUST_PROXIES_IP_RANGE_LIST")

sed -e "s|\${ROOT_DOMAIN}|${ROOT_DOMAIN}|g" \
    -e "s|\${TRAEFIK_CERT_FILENAME}|${TRAEFIK_CERT_FILENAME}|g" \
    -e "s|\${TRAEFIK_CERT_KEY_FILENAME}|${TRAEFIK_CERT_KEY_FILENAME}|g" \
    -e "s|\${TRAEFIK_DASHBOARD_USERNAME}|${TRAEFIK_DASHBOARD_USERNAME}|g" \
    -e "s|\${TRAEFIK_DASHBOARD_PASSWORD}|${TRAEFIK_DASHBOARD_PASSWORD}|g" \
    -e "s|\${TRAEFIK_WHITELIST_IP_RANGE_LIST}|${TRAEFIK_IPS}|g" \
    -e "s|\${ADMIN_DASHBOARD_WHITELIST_IP_RANGE_LIST}|${ADMIN_IPS}|g" \
    env/template/traefik-dynamic.yaml.template > source/traefik/dynamic.yaml

sed -e "s|\${TRAEFIK_TRUST_PROXIES_IP_RANGE_LIST}|${PROXY_WHITELIST_IPS}|g" \
    env/template/traefik.yaml.template > source/traefik/traefik.yaml

echo "configuring .env"

echo "# ignore this" > .env
echo "UID=$(id -u)" >> .env
echo "# ignore this" >> .env
echo "GID=$(id -g)" >> .env
echo "" >> .env
echo "ROOT_DOMAIN=$ROOT_DOMAIN" >> .env
echo "FALLBACK_URL=$FALLBACK_URL" >> .env
echo "LOG_RETENTION_DAYS=$LOG_RETENTION_DAYS" >> .env
echo "DB_PASSWORD=$DB_PASSWORD" >> .env
echo "ANALYTIC_IPINFO_TOKEN=$ANALYTIC_IPINFO_TOKEN" >> .env
