#!/bin/sh

set -e

rm -rf /usr/share/nginx/html/*
cp -r /usr/local/src/ferriskey/* /usr/share/nginx/html
envsubst < /usr/local/src/ferriskey/config.json > /usr/share/nginx/html/config.json
