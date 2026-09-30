import { deriveCodeChallenge, generateCodeVerifier, storeOAuthFlow } from '../utils/pkce.ts'

export const CONSOLE_CLIENT_ID = 'security-admin-console'

export type ProviderLoginUrlInput = {
  loginUrl: string
  apiUrl: string
  currentSearch: string
  origin: string
  pathname: string
}

const isAbsoluteUrl = (value: string) => /^https?:\/\//i.test(value)

const currentRealm = (pathname: string) => pathname.match(/\/realms\/([^/]+)/)?.[1] ?? 'master'

/**
 * Builds the `/broker/{alias}/login` URL behind an identity provider button.
 *
 * When the login page was reached from a third-party application's
 * `/authorize`, its `client_id`, `redirect_uri` and `state` are forwarded
 * untouched: the broker echoes that state back to the application, which
 * alone holds the matching PKCE verifier. Otherwise the console owns the
 * flow and gets a fresh state (registered for `PageCallbackFeature`) and an
 * S256 challenge whose verifier is kept via `storeOAuthFlow`.
 *
 * Rejects when `crypto.subtle` is unavailable (non-secure context).
 */
export const buildProviderLoginUrl = async ({
  loginUrl,
  apiUrl,
  currentSearch,
  origin,
  pathname,
}: ProviderLoginUrlInput): Promise<string> => {
  const base = apiUrl.endsWith('/') ? apiUrl : `${apiUrl}/`
  const path = loginUrl.replace(/^\//, '')
  const url = new URL(isAbsoluteUrl(loginUrl) ? loginUrl : path, base)
  const currentParams = new URLSearchParams(currentSearch)

  currentParams.forEach((value, key) => {
    if (!url.searchParams.has(key)) {
      url.searchParams.set(key, value)
    }
  })

  // An application's state and PKCE binding are its own: the broker echoes that
  // state back to the application, which alone holds the matching verifier.
  const clientId = url.searchParams.get('client_id')
  if (clientId && clientId !== CONSOLE_CLIENT_ID) {
    return url.toString()
  }

  // Same rule as useOAuthParams: never trust a redirect_uri carried over from another realm.
  const redirectUri = `${origin}/realms/${currentRealm(pathname)}/authentication/callback`

  const state = crypto.randomUUID()
  const codeVerifier = generateCodeVerifier()
  const codeChallenge = await deriveCodeChallenge(codeVerifier)
  // The callback page rejects any state it did not see issued here.
  localStorage.setItem(`oauth_state:${state}`, state)
  storeOAuthFlow(state, { codeVerifier, redirectUri })

  // A reused auth session keeps its original challenge, which this verifier cannot satisfy.
  url.searchParams.delete('session_id')
  url.searchParams.set('client_id', CONSOLE_CLIENT_ID)
  url.searchParams.set('redirect_uri', redirectUri)
  url.searchParams.set('response_type', 'code')
  url.searchParams.set('state', state)
  url.searchParams.set('code_challenge', codeChallenge)
  url.searchParams.set('code_challenge_method', 'S256')

  return url.toString()
}
