import { useCallback, useMemo } from 'react'
import { useLocation, useParams } from 'react-router'

export function useOAuthParams() {
  const { realm_name } = useParams()
  const location = useLocation()
  const searchParams = useMemo(() => new URLSearchParams(location.search), [location.search])
  const clientId = searchParams.get('client_id')
  const redirectUri = searchParams.get('redirect_uri')
  const loginError = searchParams.get('login_error')

  const currentRealm = realm_name ?? 'master'
  const realmCallbackUri = `${window.location.origin}/realms/${currentRealm}/authentication/callback`

  // Detect stale realm in redirect_uri.
  // When the user edits the realm in the URL (e.g. /realms/master → /realms/cloud-iam)
  // the old redirect_uri (".../realms/master/authentication/callback") stays in the query
  // params. If we don't catch this, the OAuth session is created with the wrong callback
  // and the user ends up redirected to the wrong realm after login.
  const isRedirectUriStale =
    redirectUri !== null &&
    redirectUri.includes('/realms/') &&
    !redirectUri.includes(`/realms/${currentRealm}/`)

  const isAuthInitiated = Boolean(clientId && redirectUri) && !isRedirectUriStale

  const getAuthParamsFromUrl = useCallback(() => {
    const resolvedClientId = clientId ?? 'security-admin-console'
    // For the webapp's own OAuth client, always derive redirect_uri from the
    // current realm — never use a stale value carried over from another realm's
    // query params.
    const resolvedRedirectUri =
      resolvedClientId === 'security-admin-console'
        ? realmCallbackUri
        : (redirectUri ?? realmCallbackUri)
    return {
      clientId: resolvedClientId,
      redirectUri: resolvedRedirectUri,
    }
  }, [clientId, redirectUri, realmCallbackUri])

  const getOAuthParams = useCallback(() => {
    const state = crypto.randomUUID()
    localStorage.setItem(`oauth_state:${state}`, state)
    const { clientId, redirectUri } = getAuthParamsFromUrl()

    return {
      query: new URLSearchParams({
        response_type: 'code',
        client_id: clientId,
        redirect_uri: redirectUri,
        scope: 'openid profile email',
        state,
      }).toString(),
      realm: currentRealm,
    }
  }, [getAuthParamsFromUrl, currentRealm])

  return {
    realm_name,
    isAuthInitiated,
    loginError,
    getAuthParamsFromUrl,
    getOAuthParams,
  }
}
