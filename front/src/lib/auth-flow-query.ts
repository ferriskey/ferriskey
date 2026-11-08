/**
 * The query an application's `/authorize` puts on the sign-in page: which client the user is
 * signing in to, where to send them back, and the application's own `state`.
 *
 * Pages that link to one another inside one sign-in (login, register, forgot password, check your
 * email) must carry it along. A link without it reaches a sign-in page that knows no client and
 * falls back to the console's own client, so the user ends up in the console instead of back in the
 * application they came from (and on `invalid_client` when the realm's console client is disabled).
 */
export const AUTH_FLOW_PARAMS = ['client_id', 'redirect_uri', 'state'] as const

/** `?client_id=…&redirect_uri=…&state=…` from `currentSearch`, or `''` when none is present. */
export const authFlowSearch = (currentSearch: string): string => {
  const current = new URLSearchParams(currentSearch)
  const kept = new URLSearchParams()

  for (const name of AUTH_FLOW_PARAMS) {
    const value = current.get(name)
    if (value !== null && value !== '') kept.set(name, value)
  }

  const query = kept.toString()
  return query ? `?${query}` : ''
}

/** `path` with the sign-in query of `currentSearch` appended. */
export const withAuthFlow = (path: string, currentSearch: string): string =>
  `${path}${authFlowSearch(currentSearch)}`

/**
 * Whether `currentSearch` carries enough to sign in to an application: its `client_id` and the
 * `redirect_uri` to return to. With `client_id` alone the sign-in page would use the console's
 * callback as the redirect URI, which the application's client does not accept.
 */
export const hasAuthFlowClient = (currentSearch: string): boolean => {
  const current = new URLSearchParams(currentSearch)
  return Boolean(current.get('client_id') && current.get('redirect_uri'))
}
