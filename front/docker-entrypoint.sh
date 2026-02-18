#!/bin/sh

set -eu

HTML_DIR=/usr/share/angie/html
SRC_DIR=/usr/local/src/ferriskey
CONFIG_FILE="$HTML_DIR/config.json"

rm -rf -- "${HTML_DIR:?}/"* "${HTML_DIR:?}/".[!.]* "${HTML_DIR:?}/"..?*
cp -r "$SRC_DIR"/* "$HTML_DIR"

if [ -f "$CONFIG_FILE" ]; then
  api_url="${API_URL:-}"
  escaped_api_url=$(printf '%s' "$api_url" | sed -e 's/[\/&]/\\&/g')
  # shellcheck disable=SC2016
  placeholder='${API_URL}'
  sed -i "s|$placeholder|$escaped_api_url|g" "$CONFIG_FILE"
fi

exec "$@"
