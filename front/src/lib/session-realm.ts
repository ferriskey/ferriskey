/**
 * The realm named by a token issuer (`https://auth.example.com/realms/master`
 * → `master`).
 *
 * The console is a client of the realm the admin signed in to, but it browses
 * every realm that admin may manage, so the realm in the page URL is not the
 * session's realm. Token-endpoint calls (refresh, revoke, logout) and the
 * sign-in page belong to the session's realm: sent to the browsed realm they
 * reach another realm's `security-admin-console`, which does not know the
 * session and answers `invalid_client` when it is disabled.
 *
 * Realm names are limited to `[A-Za-z0-9_-]` and the issuer is built from the
 * name unencoded, so the segment is taken as it stands.
 */
export function realmFromIssuer(iss: string | null | undefined): string | null {
  if (!iss) return null
  return iss.match(/\/realms\/([A-Za-z0-9_-]+)\/?$/)?.[1] ?? null
}
