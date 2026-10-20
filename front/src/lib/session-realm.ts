/**
 * The realm a console session belongs to, read from the `iss` claim of one of
 * its tokens (`https://auth.example.com/realms/master` → `master`).
 *
 * The console is a client of the realm the admin signed in to, but it browses
 * every realm that admin may manage, so the realm in the page URL is not the
 * session's realm. Token-endpoint calls (refresh, revoke, logout) and the
 * sign-in page belong to the session's realm: sent to the browsed realm they
 * reach another realm's `security-admin-console`, which does not know the
 * session and answers `invalid_client` when it is disabled.
 *
 * Only the payload is decoded, never verified: the value picks an endpoint,
 * and the server still validates the token it receives there.
 */
export function realmFromToken(token: string | null | undefined): string | null {
  if (!token) return null

  const payload = token.split('.')[1]
  if (!payload) return null

  let iss: unknown
  try {
    const base64 = payload.replace(/-/g, '+').replace(/_/g, '/')
    const padded = base64.padEnd(Math.ceil(base64.length / 4) * 4, '=')
    iss = (JSON.parse(atob(padded)) as Record<string, unknown>).iss
  } catch {
    return null
  }
  if (typeof iss !== 'string') return null

  const match = iss.match(/\/realms\/([^/?#]+)\/?$/)
  if (!match) return null

  try {
    return decodeURIComponent(match[1])
  } catch {
    return null
  }
}

/**
 * The session's realm from whichever token is present, falling back to the
 * realm in the URL when neither carries one (no session yet, or a malformed
 * token).
 */
export function sessionRealm(
  accessToken: string | null | undefined,
  refreshToken: string | null | undefined,
  fallback: string
): string {
  return realmFromToken(accessToken) ?? realmFromToken(refreshToken) ?? fallback
}
