# Moving a realm to another instance

A realm can be exported from one FerrisKey instance and imported on another
with every id kept: the realm id, user ids (the `sub` of every token), clients,
roles, signing keys and their `kid`, sessions and refresh tokens. Tokens issued
before the move keep validating, refresh tokens keep refreshing, and the SSO
cookie keeps skipping the login page.

## Before you start

- FerrisKey does not freeze the realm during the move yet (#1595). Stop the
  traffic to the realm at your reverse proxy before the export, and keep it
  stopped until the proxy points at the target. Anything written on the
  source after the export is lost.
- The public URL of the realm must not change. Every token carries the issuer
  `https://<public host>/realms/<realm>`; the switch is done at the proxy, not
  in FerrisKey (per-realm public URLs are #1401).
- The target must run the same FerrisKey version as the source, or a newer
  one. A bundle from a newer schema is refused.
- The `master` realm cannot be moved.
- Both calls need an administrator of the `master` realm who can create
  realms, and who manages the exported realm.
- Pick a passphrase of 16 to 1024 printable ASCII characters and keep it out
  of shell history. Whoever holds the file and the passphrase holds the
  realm's signing keys and password hashes.

## Procedure

1. Stop the traffic to the realm at the proxy.
2. Export:

   ```sh
   curl -sS -X POST "https://source.example/realms/acme/export" \
     -H "Authorization: Bearer $ADMIN_TOKEN" \
     -H "X-Ferriskey-Bundle-Passphrase: $PASSPHRASE" \
     -o acme.fkrb
   ```

   The body is streamed. A file cut short is refused at import, so a broken
   connection never imports half a realm.
3. Import on the target:

   ```sh
   curl -sS -X POST "https://target.example/realms/import" \
     -H "Authorization: Bearer $TARGET_ADMIN_TOKEN" \
     -H "X-Ferriskey-Bundle-Passphrase: $PASSPHRASE" \
     -H "Content-Type: application/vnd.ferriskey.realm-bundle" \
     --data-binary @acme.fkrb
   ```

   `201` means the realm was created, `200` that it existed and was
   overwritten. The response lists the rows written per table, and the columns
   the target ignored because a later migration dropped them.
4. Point the proxy route of the realm at the target.
5. Once the target serves the realm, delete it on the source and delete the
   bundle file.

The import is one transaction. A wrong passphrase, a tampered or truncated
file, a bundle without an active signing key, or a row that clashes with
another realm of the target (`409`) leaves the target as it was. Running the
same import again is safe.

## What moves

Everything the realm holds: settings and password policy, SMTP configuration,
email templates, portal themes and layouts, clients and their URIs, SAML
configuration, roles and role mappings, client scopes and protocol mappers,
users with their password hashes, OTP and passkey credentials, attributes and
required actions, identity providers and user links, federation providers and
mappings, organizations, groups and member roles, webhooks, token exchange
policies, resource owners, the authorization model, consents, user sessions,
refresh tokens, stored access tokens and revocations, every signing key
(active, passive, retired, revoked) and the audit trail.

What does not move, because it is in flight and dies with the cut-over: login
attempts in progress (auth sessions, broker and device sessions), magic links,
password reset and email verification tokens, enrolments in progress, the
webhook delivery queue and the Compass traces.

Secrets (signing keys, OTP seeds, SMTP password) are opened with the source
`SECRETS_ENCRYPTION_KEYS` and sealed again with the target's. The two
instances do not need to share a key ring.

## File format

The file is meant to be read by tools other than FerrisKey (`ferris-ctl`).

The envelope is a 41-byte header followed by frames:

| Offset | Size | Field |
|---|---|---|
| 0 | 4 | `FKRB` |
| 4 | 1 | envelope version, `1` |
| 5 | 1 | key derivation, `1` = Argon2id |
| 6 | 4 | Argon2 memory in KiB, big endian |
| 10 | 4 | Argon2 passes, big endian |
| 14 | 4 | Argon2 lanes, big endian |
| 18 | 16 | salt |
| 34 | 7 | nonce prefix |

The key is Argon2id(passphrase, salt), 32 bytes. Each frame is a 4-byte
big-endian length and an AES-256-GCM ciphertext with its tag, at most 64 KiB of
plaintext. Frame `n` uses the nonce `prefix ‖ n (u32, big endian) ‖ last`,
where `last` is `1` on the final frame only, and the 41 header bytes as
associated data.

The plaintext is JSON Lines: a header line, one line per row, a trailer line.

```json
{"kind":"header","format":"ferriskey.realm-bundle","version":{"major":1,"minor":0},"realm":{"id":"…","name":"acme"},"schema":20261031181100,"exported_at":"…"}
{"kind":"row","table":"users","row":{"id":"…","realm_id":"…","username":"alice"}}
{"kind":"trailer","rows":{"realms":1,"jwt_keys":2,"users":1}}
```

A row holds one key per database column. Tables come in a fixed order where a
row only points at rows above it, with `realms` first and the signing keys
right after. `major` changes when an older reader would misread the file;
`minor` changes when the set of tables or sealed columns changes. New columns
need no version change.
