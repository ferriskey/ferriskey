#!/bin/sh
set -eu

if [ -z "${DATABASE_URL:-}" ]; then
  DATABASE_URL="postgresql://${DATABASE_USER:-ferriskey}:${DATABASE_PASSWORD:-ferriskey}@${DATABASE_HOST:-localhost}:${DATABASE_PORT:-5432}/${DATABASE_NAME:-ferriskey}"
fi
export DATABASE_URL

sqlx migrate run --source /usr/local/src/ferriskey/migrations

CONFIG_FILE="/usr/share/nginx/html/config.json"
if [ -f "$CONFIG_FILE" ]; then
  api_url="${API_URL:-}"
  escaped_api_url=$(printf '%s' "$api_url" | sed -e 's/[\/&|]/\\&/g')
  # shellcheck disable=SC2016
  sed -i "s|\${API_URL}|$escaped_api_url|g" "$CONFIG_FILE"
fi

exec /usr/bin/supervisord -c /etc/supervisor/conf.d/ferriskey.conf
