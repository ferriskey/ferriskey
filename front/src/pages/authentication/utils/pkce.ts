// PKCE (RFC 7636) for the admin console, which is a public client: it has no
// secret, so the code_verifier is the only thing proving that the browser
// redeeming an authorization code is the one that requested it.

const FLOW_STORAGE_PREFIX = 'oauth_flow:'

type OAuthFlow = {
  codeVerifier: string
  // Kept alongside the verifier so the value sent to the token endpoint is
  // byte-identical to the one sent to /auth, which the server now requires.
  redirectUri: string
}

function base64UrlEncode(bytes: Uint8Array): string {
  let binary = ''
  for (const byte of bytes) {
    binary += String.fromCharCode(byte)
  }
  return btoa(binary).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '')
}

// RFC 7636 §4.1: 43–128 chars from the unreserved set. 32 random bytes
// base64url-encoded yields exactly 43.
export function generateCodeVerifier(): string {
  const bytes = new Uint8Array(32)
  crypto.getRandomValues(bytes)
  return base64UrlEncode(bytes)
}

// RFC 7636 §4.2: BASE64URL(SHA256(ASCII(code_verifier))).
// crypto.subtle needs a secure context, so the console must be served over
// HTTPS (or localhost).
export async function deriveCodeChallenge(codeVerifier: string): Promise<string> {
  const digest = await crypto.subtle.digest('SHA-256', new TextEncoder().encode(codeVerifier))
  return base64UrlEncode(new Uint8Array(digest))
}

export function storeOAuthFlow(state: string, flow: OAuthFlow): void {
  try {
    sessionStorage.setItem(`${FLOW_STORAGE_PREFIX}${state}`, JSON.stringify(flow))
  } catch {
    // Storage unavailable (private mode, quota): the token exchange will fail
    // closed on a missing verifier rather than silently dropping PKCE.
  }
}

// Reads and removes the flow record: a verifier is single-use, like the code.
export function takeOAuthFlow(state: string | null): OAuthFlow | null {
  if (!state) return null
  const key = `${FLOW_STORAGE_PREFIX}${state}`
  try {
    const raw = sessionStorage.getItem(key)
    sessionStorage.removeItem(key)
    if (!raw) return null
    const parsed: unknown = JSON.parse(raw)
    if (
      typeof parsed === 'object' &&
      parsed !== null &&
      typeof (parsed as OAuthFlow).codeVerifier === 'string' &&
      typeof (parsed as OAuthFlow).redirectUri === 'string'
    ) {
      return parsed as OAuthFlow
    }
    return null
  } catch {
    return null
  }
}
