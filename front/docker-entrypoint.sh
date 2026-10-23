#!/bin/sh

set -eu

CONFIG_FILE=/usr/share/nginx/html/config.json
# shellcheck disable=SC2016
PLACEHOLDER='${API_URL}'

# The Helm chart mounts its own config.json from a ConfigMap, without the
# placeholder; API_URL only applies to the config.json shipped in the image.
if [ -n "${API_URL:-}" ] && grep -qF "$PLACEHOLDER" "$CONFIG_FILE"; then
  escaped_api_url=$(printf '%s' "$API_URL" | sed -e 's/[\/&|]/\\&/g')
  content=$(sed "s|$PLACEHOLDER|$escaped_api_url|g" "$CONFIG_FILE")
  printf '%s\n' "$content" > "$CONFIG_FILE"
fi

NGINX_CONF=/etc/nginx/conf.d/default.conf
# shellcheck disable=SC2016
CSP_PLACEHOLDER='${CSP_CONNECT_SRC}'

if grep -qF "$CSP_PLACEHOLDER" "$NGINX_CONF"; then
  api_origin=$(printf '%s' "${API_URL:-}" | sed -nE 's#^(https?://[^/?\#]+).*$#\1#p')
  escaped_origin=$(printf '%s' "$api_origin" | sed -e 's/[\/&|]/\\&/g')
  content=$(sed "s|$CSP_PLACEHOLDER|$escaped_origin|g" "$NGINX_CONF")
  printf '%s\n' "$content" > "$NGINX_CONF"
fi
