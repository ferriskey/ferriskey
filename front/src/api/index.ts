import { toast } from 'sonner'
import { type Fetcher } from '@/api/api.client.ts'
import { authRefreshController } from '@/hooks/auth-refresh-controller.ts'
import { authStore } from '@/store/auth.store.ts'

export interface BaseQuery {
  realm?: string
}

/**
 * The auth refresh endpoint itself — we must NEVER intercept a 401 from
 * this URL with another refresh attempt, that would loop forever. The
 * fetcher checks for this substring before kicking off a refresh.
 */
const TOKEN_ENDPOINT_SEGMENT = '/protocol/openid-connect/token'

/**
 * Pull the realm name out of an API URL like
 * `https://api.example.com/realms/master/users/123`. Used by the 401
 * handler to know which realm's token endpoint to hit for the refresh.
 */
function extractRealm(url: URL): string | null {
  const match = url.pathname.match(/\/realms\/([^/]+)\//)
  return match?.[1] ?? null
}

/**
 * Call the realm's token endpoint with a `grant_type=refresh_token` body.
 * Uses plain `fetch` instead of our wrapped fetcher to avoid recursing
 * into the same 401 handler if the refresh itself fails.
 */
async function refreshSession(
  baseUrl: URL,
  realm: string,
  refreshToken: string,
): Promise<{ access_token: string; refresh_token: string; id_token?: string }> {
  const tokenUrl = new URL(`/realms/${realm}${TOKEN_ENDPOINT_SEGMENT}`, baseUrl)
  const body = new URLSearchParams({
    grant_type: 'refresh_token',
    client_id: 'security-admin-console',
    refresh_token: refreshToken,
  })
  const response = await fetch(tokenUrl, {
    method: 'POST',
    headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
    body,
    credentials: 'include',
  })
  if (!response.ok) {
    throw new Error(`refresh failed: HTTP ${response.status}`)
  }
  return response.json()
}

export const fetcher: Fetcher['fetch'] = async (input) => {
  const buildHeaders = (token: string | null): Headers => {
    const h = new Headers()
    if (token) h.set('Authorization', `Bearer ${token}`)
    if (input.parameters?.header) {
      Object.entries(input.parameters.header).forEach(([key, value]) => {
        if (value != null) h.set(key, String(value))
      })
    }
    return h
  }

  // Handle query parameters via URLSearchParams
  if (input.urlSearchParams) {
    input.url.search = input.urlSearchParams.toString()
  }

  // Handle request body for mutation methods
  let body: BodyInit | undefined
  let contentType: string | null = null
  if (
    ['post', 'put', 'patch', 'delete'].includes(input.method.toLowerCase()) &&
    input.parameters?.body !== undefined
  ) {
    const bodyData = input.parameters.body
    if (
      bodyData instanceof URLSearchParams ||
      bodyData instanceof FormData ||
      typeof bodyData === 'string' ||
      bodyData instanceof Blob ||
      bodyData instanceof ArrayBuffer
    ) {
      body = bodyData as BodyInit
    } else {
      body = JSON.stringify(bodyData)
      contentType = 'application/json'
    }
  }

  const execute = async (token: string | null): Promise<Response> => {
    const headers = buildHeaders(token)
    if (contentType && !headers.has('Content-Type')) {
      headers.set('Content-Type', contentType)
    }
    return fetch(input.url, {
      method: input.method.toUpperCase(),
      ...(body && { body }),
      headers,
      credentials: 'include',
      ...input.overrides,
    })
  }

  let response = await execute(authStore.getState().accessToken)

  // 401 handling: try refreshing the session once and replay the original
  // request with the new access token. Skip the dance when:
  //   - the request that 401'd was the token endpoint itself (refresh of a
  //     refresh would loop forever)
  //   - we have no refresh token to spend
  //   - we can't figure out the realm from the URL (extremely unlikely —
  //     every API endpoint lives under /realms/<name>/)
  // The shared `authRefreshController` de-dupes concurrent attempts so a
  // page that fires 12 parallel queries on mount only triggers one refresh.
  const isTokenEndpoint = input.url.pathname.includes(TOKEN_ENDPOINT_SEGMENT)
  if (response.status === 401 && !isTokenEndpoint) {
    const refreshToken = authStore.getState().refreshToken
    const realm = extractRealm(input.url)
    if (refreshToken && realm) {
      try {
        const refreshed = await authRefreshController.run(realm, () =>
          refreshSession(input.url, realm, refreshToken),
        )
        authStore.getState().setTokens(
          refreshed.access_token,
          refreshed.refresh_token,
          refreshed.id_token ?? authStore.getState().idToken,
        )
        response = await execute(refreshed.access_token)
      } catch (refreshError) {
        // Refresh failed: clear local tokens so the auth-guard redirects
        // the next render to /login, and surface a toast in case the user
        // is mid-flow and would otherwise see the underlying 401 silently.
        // The original 401 still throws below to abort the in-flight call.
        if (
          !(refreshError instanceof Error) ||
          refreshError.message !== 'refresh temporarily blocked'
        ) {
          authStore.getState().setTokens(null, null, null)
          toast.error('Your session expired. Please sign in again.', {
            action: {
              label: 'Sign in',
              onClick: () => {
                window.location.href = `/realms/${realm}/authentication/login`
              },
            },
          })
        }
      }
    }
  }

  if (!response.ok) {
    let errorBody: Record<string, unknown> | undefined
    try {
      errorBody = await response.json()
    } catch {
      // If parsing fails, errorBody stays undefined
    }

    const error: Error & { status?: number; data?: Record<string, unknown> } = new Error(
      (errorBody?.message as string) ?? `HTTP ${response.status}: ${response.statusText}`,
    )
    error.status = response.status
    error.data = errorBody
    throw error
  }

  return response
}
