#!/usr/bin/env bash
# Builds every test binary of the workspace in one cargo invocation, then runs
# the binaries concurrently with their ignored (database-backed) tests
# included. Each binary boots its own Postgres schema, so binaries share no
# state; the tests inside one binary keep sharing their fixture, as they do
# under `cargo test`.
set -euo pipefail

jobs="${TEST_JOBS:-$(getconf _NPROCESSORS_ONLN)}"
logs="$(mktemp -d)"
export logs

# Ignored for a reason other than needing Postgres: they assert an open
# finding, or have rotted while CI never ran them. Remove an entry once fixed.
skip=(
  sso_08_parallel_refreshes_keep_the_session_alive
  sso_11_host_header_does_not_change_the_issuer
  creates_client_with_realm_default_scopes_assigned
)
export SKIP_ARGS="${skip[*]/#/--skip }"

cargo test --workspace --exclude ferriskey-client --lib --bins --tests --locked --no-run \
  --message-format=json-render-diagnostics \
  | jq -r 'select(.reason == "compiler-artifact" and .executable != null)
           | select(.profile.test or (.target.kind | index("test")))
           | [.target.name, .executable, .manifest_path] | @tsv' \
  > "$logs/binaries.tsv"

run() {
  local name="$1" exe="$2" dir log
  dir="$(dirname "$3")"
  log="$logs/$(basename "$exe").log"
  # shellcheck disable=SC2206
  local args=(--include-ignored $SKIP_ARGS)
  case "$name" in
    # `harness = false`: parses its own arguments and rejects libtest flags.
    oidc_bdd) args=() ;;
    # Shares one realm whose CORS origin cache is keyed by realm name alone,
    # so its tests race when run concurrently.
    web_origin_test) args+=(--test-threads=1) ;;
  esac

  local start=$SECONDS
  if (cd "$dir" && CARGO_MANIFEST_DIR="$dir" "$exe" ${args[@]+"${args[@]}"}) >"$log" 2>&1; then
    echo "ok   $name ($((SECONDS - start))s) $(grep -h '^test result:' "$log" | tail -1)"
  else
    echo "FAIL $name ($((SECONDS - start))s)"
    echo "$log" >>"$logs/failed"
  fi
}
export -f run

# Assert how many statements a warm request sends, so a cache emptied by another
# binary's token or session change (the NOTIFY channel is shared by the whole
# database, not by schema) turns into a failure. They run alone, after the rest.
alone='^login_statement_budget_test'
grep -Ev "$alone" "$logs/binaries.tsv" | xargs -n 3 -P "$jobs" bash -c 'run "$@"' _
alone_binaries="$(grep -E "$alone" "$logs/binaries.tsv" || true)"
if [[ -n "$alone_binaries" ]]; then
  xargs -n 3 -P 1 bash -c 'run "$@"' _ <<<"$alone_binaries"
fi

if [[ -s "$logs/failed" ]]; then
  while read -r log; do
    echo "::group::$(basename "$log")"
    cat "$log"
    echo "::endgroup::"
  done <"$logs/failed"
  exit 1
fi
