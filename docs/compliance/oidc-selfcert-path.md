# OIDC Self-Certification Path

This document describes how to submit FerrisKey conformance results to the
OpenID Foundation (OIDF) self-certification programme once the identified gaps
have been resolved.

---

## What self-certification is

OIDF self-certification is a self-attestation programme: you run the OIDF-hosted
conformance suite, upload the results, and the OIDF lists your software on
<https://openid.net/certification/>. It is distinct from third-party certification
(which requires a formal evaluation). Self-certification is free and carries no
compliance audit.

---

## Profiles available for an OP (OpenID Provider)

| Profile | Description | Priority |
|---|---|---|
| **Basic OP** | authorization\_code flow + ID token + UserInfo | Start here |
| **Config OP** | Discovery endpoint completeness | Required for most deployments |
| **Hybrid OP** | `code token` / `code id_token` response types | Not yet applicable |
| **Implicit OP** | `token` / `id_token` response types | Not applicable (deprecated) |
| **RP-Initiated Logout OP** | RP-Initiated Logout 1.0 | After logout improvements |
| **FAPI 1.0 Advanced OP** | Financial-grade API | Future / post-gap resolution |

Start with **Basic OP** and **Config OP** together — the OIDF requires both to
be passing for a base listing.

---

## Prerequisites for submission

Before submitting:

1. All gaps marked as blocking for Basic OP and Config OP in `oidc-gap-triage.md`
   must be resolved: GAP-01 (PKCE), GAP-02 (nonce in ID token), GAP-04 (discovery
   fields), GAP-05 (nonce capture), GAP-06 (redirect URI validation).
2. The conformance suite must pass all test modules in the chosen plan(s) with
   no FAILURE results. WARNING results may be accepted for optional features.
3. A public HTTPS endpoint for FerrisKey must be available at the time of the
   submission run (the OIDF server makes live outbound calls).

---

## Submission procedure

### Step 1 — Run the suite on the OIDF server

Follow `oidc-conformance-runbook.md`. Use the hosted server at
<https://www.certification.openid.net/> (not the local Docker option) — only
results generated on the OIDF-hosted server are accepted for self-certification.

### Step 2 — Export the result log

On the test plan summary page:

1. Confirm every test module shows PASSED (green).
2. Click **Export** → download the ZIP.
3. Note the **plan ID** shown in the URL (e.g. `plan-a1b2c3d4`).

### Step 3 — Prepare the submission package

The OIDF requires:

- The exported JSON result log (from the ZIP).
- A completed **self-certification form** (available at
  <https://openid.net/certification/op-certification/>).
  Key fields:
  - Software name: `FerrisKey`
  - Software version: the release tag (e.g. `v0.8.0`)
  - Conformance profile(s): `Basic OP`, `Config OP`
  - Test plan IDs: the plan IDs from Step 2
  - Contact email: `security@ferriskey.rs` (or maintainer address)
  - Open-source URL: `https://github.com/ferriskey/ferriskey`

### Step 4 — Submit

Email the completed form and result logs to `certification@oidf.org` or use the
web submission at <https://openid.net/certification/op-certification/>.

The OIDF typically processes submissions within a few business days. Once
approved, FerrisKey will appear on the certified implementations page.

---

## Where to keep evidence in the repository

```
conformance/
  results/               # gitignored — store per-run ZIPs locally
  basic-certification.json   # test plan config template (tracked)
docs/compliance/
  oidc-conformance-runbook.md
  oidc-support-matrix.md
  oidc-gap-triage.md
  oidc-selfcert-path.md    # this file
```

Add `conformance/results/` to `.gitignore` to avoid committing large JSON logs.
If you want a specific passing run as a permanent record, extract the summary
section and commit it as `conformance/results/summary-<version>.json`.

---

## Renewal

Self-certification listings on the OIDF site are version-specific. Each new
major or minor release that changes the OAuth/OIDC behaviour should trigger a
fresh conformance run and submission. Add a checklist item to the release process
in `CONTRIBUTING.md`:

```
- [ ] Run OIDF conformance suite (Basic OP + Config OP)
- [ ] All modules PASS
- [ ] Update conformance/results/summary-<version>.json
- [ ] Submit to OIDF if significant protocol changes were made
```

---

## References

- OIDF Certification Programme: <https://openid.net/certification/>
- Self-Certification How-To: <https://openid.net/certification/op-certification/>
- Conformance Suite Repository: <https://gitlab.com/openid/conformance-suite>
- Conformance Suite Hosted: <https://www.certification.openid.net/>
- OpenID Connect Core 1.0: <https://openid.net/specs/openid-connect-core-1_0.html>
- RFC 8414 (OAuth 2.0 Authorization Server Metadata): <https://www.rfc-editor.org/rfc/rfc8414>
- RFC 7636 (PKCE): <https://www.rfc-editor.org/rfc/rfc7636>
- RFC 7009 (Token Revocation): <https://www.rfc-editor.org/rfc/rfc7009>
- RFC 7662 (Token Introspection): <https://www.rfc-editor.org/rfc/rfc7662>
