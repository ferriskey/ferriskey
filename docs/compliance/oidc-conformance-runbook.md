# OIDC Conformance Suite Runbook

This runbook describes how to run the OpenID Foundation (OIDF) conformance suite
against a FerrisKey instance and capture the results needed for self-certification.

---

## Prerequisites

| Requirement | Notes |
|---|---|
| FerrisKey instance reachable over HTTPS | Self-signed certs are accepted by the suite if you set the flag below |
| Publicly reachable base URL (or local tunnel) | The OIDF server makes outbound calls to your instance |
| A realm created and an OIDC client registered | See [Client registration](#2-register-a-conformance-client) |
| A test user account in that realm | Used by the interactive authorization\_code flow tests |
| Node.js ≥ 18 **or** Docker (for the local conformance server option) | See [Suite options](#suite-options) |

---

## Suite options

**Option A — OIDF hosted server (recommended)**
Use <https://www.certification.openid.net/>. No local install required. Results
are stored on the OIDF server and can be used directly for self-certification
evidence.

**Option B — Self-hosted suite**

```bash
git clone https://gitlab.com/openid/conformance-suite.git
cd conformance-suite
# Requires Docker Compose
docker-compose up
# Server available at http://localhost:8080
```

The remaining steps are the same for both options.

---

## Step 1 — Configure the FerrisKey instance

FerrisKey must serve its discovery document at the correct well-known URL.
Verify it is reachable before starting:

```bash
# Replace <base_url> and <realm> with your values
curl https://<base_url>/realms/<realm>/.well-known/openid-configuration | jq .
```

Expected fields at minimum: `issuer`, `authorization_endpoint`,
`token_endpoint`, `jwks_uri`, `userinfo_endpoint`.

**Environment variables for the API instance:**

```bash
PORT=3333
ALLOWED_ORIGINS=https://<base_url>
# Ensure the instance is exposed over TLS (use a reverse proxy if needed)
```

---

## Step 2 — Register a conformance client

Use the FerrisKey admin console or API to create an **OpenID Connect** client
in your test realm with the following settings:

| Field | Value |
|---|---|
| Client ID | `oidf-conformance-client` (or any name you choose) |
| Client secret | generate a strong random secret |
| Access type | Confidential |
| Redirect URIs | `https://www.certification.openid.net/test/a/<YOUR_TEST_ALIAS>/callback` |
| Post-logout redirect URIs | `https://www.certification.openid.net/test/a/<YOUR_TEST_ALIAS>/post-logout-redirect` |
| Standard flow (authorization\_code) | Enabled |
| Direct access grants (password) | Enabled (required for some test variants) |
| Grant types | `authorization_code`, `refresh_token`, `client_credentials` |
| Response types | `code` |
| Scopes | `openid`, `profile`, `email` |

> Replace `<YOUR_TEST_ALIAS>` with the alias you enter when you create the test
> plan on the OIDF server.

---

## Step 3 — Create a test plan on the OIDF server

1. Go to <https://www.certification.openid.net/> and sign in (or use the local
   server at `http://localhost:8080`).
2. Click **Create a new test plan**.
3. Select the plan appropriate to the profiles you want to certify:
   - **Basic OP** — the baseline profile (recommended first run).
   - **Config OP** — tests the discovery / metadata endpoints.
   - **Hybrid OP**, **Implicit OP** — only if those flows are needed.
4. Set the **Alias** (used in redirect URI construction above).
5. Fill in the **Server metadata** section:

```json
{
  "server": {
    "discoveryUrl": "https://<base_url>/realms/<realm>/.well-known/openid-configuration"
  },
  "client": {
    "client_id": "oidf-conformance-client",
    "client_secret": "<your_client_secret>"
  },
  "resource": {
    "resourceUrl": "https://<base_url>/realms/<realm>/protocol/openid-connect/userinfo"
  }
}
```

6. Click **Create test plan**.

---

## Step 4 — Run the tests

The suite presents individual test modules. Run them in order:

1. **oidcc-server-rotate-keys** — JWKS rotation (can skip if key rotation not yet implemented).
2. **oidcc-discovery-endpoint-test** — validates the `.well-known/openid-configuration` response.
3. **oidcc-basic-*** — the authorization\_code flow tests.
4. **oidcc-userinfo-*** — userinfo endpoint tests.
5. **oidcc-logout-*** — RP-Initiated Logout tests.

For tests that require user interaction (login page), the suite will pause and
display a URL to open in a browser. Complete the login as the test user you
created, then return to the suite.

---

## Step 5 — Capture and export results

After all modules in a test plan have run:

1. Click **Export** on the test plan summary page.
2. Download the JSON export (`conformance-suite-results-<alias>.zip`).
3. Store the export in `conformance/results/` in the repository (gitignored by
   default — add only if you want to check in evidence).

The JSON log contains the test plan ID, timestamp, and pass/fail per module.
This is the artefact you upload when submitting for self-certification.

---

## Step 6 — Re-run after fixes

After addressing the gaps listed in `oidc-gap-triage.md`, re-run the full
test plan from Step 3. Each run produces a new plan ID. Keep the latest
passing run as your certification evidence.

---

## Useful endpoints reference

All endpoints are scoped under `/realms/<realm>/protocol/openid-connect/`.

| Endpoint | URL suffix |
|---|---|
| Discovery | `/.well-known/openid-configuration` (realm-level path) |
| Authorization | `/auth` |
| Token | `/token` |
| UserInfo | `/userinfo` |
| JWKS | `/jwks.json` or `/certs` |
| Introspection | `/token/introspect` |
| Revocation | `/revoke` |
| End Session | `/logout` |

Full discovery URL pattern:
```
https://<base_url>/realms/<realm>/.well-known/openid-configuration
```

---

## Troubleshooting

| Symptom | Resolution |
|---|---|
| `issuer mismatch` | Verify the `host` and `x-forwarded-proto` headers reach the API correctly behind a reverse proxy |
| `nonce` claim missing from ID token | See gap `GAP-04` in `oidc-gap-triage.md` |
| `code_challenge` rejected | PKCE is not yet enforced on the inbound authorization flow; see `GAP-01` |
| `discovery` test fails on missing fields | See `GAP-05` in the gap triage doc |
| Redirect URI rejected | Ensure the URI registered in FerrisKey matches exactly; avoid relying on regex matching |
