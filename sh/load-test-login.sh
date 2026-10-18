#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Drives the password grant of a running FerrisKey and prints throughput and latency percentiles.

  sh/load-test-login.sh [options]
    --url URL            base URL of the API              (default http://127.0.0.1:3333)
    --realm NAME         realm                            (default master)
    --client-id ID       client allowed the password grant (default admin-cli)
    --username NAME      user to log in                   (default admin)
    --password SECRET    password to send                 (default admin)
    --concurrency N      concurrent connections           (default 8)
    --duration D         run duration, oha syntax         (default 20s)

Requires oha and jq. A wrong password measures the refusal path; the status code line shows it.
Use an IP rather than a host name when the machine has a resolv.conf oha cannot parse.
USAGE
}

url="${FERRISKEY_URL:-http://127.0.0.1:3333}"
realm="master"
client_id="admin-cli"
username="admin"
password="admin"
concurrency=8
duration="20s"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --url) url="$2"; shift 2 ;;
    --realm) realm="$2"; shift 2 ;;
    --client-id) client_id="$2"; shift 2 ;;
    --username) username="$2"; shift 2 ;;
    --password) password="$2"; shift 2 ;;
    --concurrency) concurrency="$2"; shift 2 ;;
    --duration) duration="$2"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) echo "unknown option: $1" >&2; usage >&2; exit 2 ;;
  esac
done

for tool in oha jq; do
  command -v "$tool" >/dev/null || { echo "$tool is required" >&2; exit 2; }
done

encode() { jq -rn --arg v "$1" '$v | @uri'; }
body="grant_type=password&client_id=$(encode "$client_id")&username=$(encode "$username")&password=$(encode "$password")&scope=openid"
endpoint="$url/realms/$realm/protocol/openid-connect/token"

oha --no-tui --output-format json -z "$duration" -c "$concurrency" \
  -m POST -H 'Content-Type: application/x-www-form-urlencoded' -d "$body" \
  "$endpoint" \
| jq -r --arg endpoint "$endpoint" --arg concurrency "$concurrency" --arg duration "$duration" '
  def ms: (. * 1000 | . * 100 | round / 100);
  "endpoint      \($endpoint)",
  "concurrency   \($concurrency)   duration \($duration)",
  "requests      \(.statusCodeDistribution | add // 0)   rps \(.summary.requestsPerSec | . * 10 | round / 10)",
  "status codes  \(.statusCodeDistribution | to_entries | map("\(.key)=\(.value)") | join("  "))",
  "errors        \(.errorDistribution // {} | to_entries | map("\(.key)=\(.value)") | join("  ") | if . == "" then "none" else . end)",
  "latency ms    p50 \(.latencyPercentiles.p50 | ms)   p95 \(.latencyPercentiles.p95 | ms)   p99 \(.latencyPercentiles.p99 | ms)   max \(.summary.slowest | ms)"
'
