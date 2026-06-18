# Conformance Configuration Templates

This directory contains configuration templates for running the
[OpenID Foundation conformance suite](https://www.certification.openid.net/)
against a FerrisKey instance.

## Files

| File | Purpose |
|---|---|
| `basic-certification.json` | Test plan config for the Basic OP + Config OP profiles |
| `results/` | Local only (gitignored) — store per-run ZIP exports here |

## Quick start

1. Copy `basic-certification.json` and replace the `<placeholder>` values:
   - `<FERRISKEY_BASE_URL>` — your FerrisKey instance hostname (HTTPS)
   - `<REALM_NAME>` — the realm to test against
   - `<CLIENT_SECRET>` — the client secret for `oidf-conformance-client`
   - `<YOUR_TEST_ALIAS>` — the alias you set when creating the plan on the OIDF server

2. Follow the full runbook at [`docs/compliance/oidc-conformance-runbook.md`](../docs/compliance/oidc-conformance-runbook.md).

3. After a passing run, consult [`docs/compliance/oidc-selfcert-path.md`](../docs/compliance/oidc-selfcert-path.md)
   for submission instructions.

## Current status

See [`docs/compliance/oidc-support-matrix.md`](../docs/compliance/oidc-support-matrix.md)
for the current PASS / PARTIAL / GAP assessment against the codebase.

Blocking gaps before a passing Basic OP run: GAP-01 (PKCE inbound), GAP-02 (nonce
in ID token), GAP-04 (discovery fields), GAP-05 (nonce capture at `/auth`).
