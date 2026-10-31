import { GrantType } from '@/api/core.interface'
import { useLogoutMutation, useRevokeTokenMutation, useTokenMutation } from '@/api/auth.api'
import type { RouterParams } from '@/routes/router'
import { authStore } from '@/store/auth.store'
import userStore from '@/store/user.store'
import { useCallback, useEffect, useState } from 'react'
import { useNavigate, useParams } from 'react-router'
import type { IUser, OrganizationClaim } from '@/contracts/states.interface'
import { translate } from '@/lib/i18n'
import { realmFromIssuer } from '@/lib/session-realm'
import { authRefreshController } from './auth-refresh-controller'

type JwtPayload = {
  exp?: number
  iss?: string
  azp?: string
  avatar?: string
  preferred_username?: string
  email?: string
  name?: string
  organizations?: Record<string, OrganizationClaim>
}

function decodeJwt(token: string): JwtPayload | null {
  try {
    const payload = token.split('.')[1]
    if (!payload) return null

    const normalizedPayload = payload.replace(/-/g, '+').replace(/_/g, '/')
    const paddedPayload = normalizedPayload.padEnd(Math.ceil(normalizedPayload.length / 4) * 4, '=')
    const decodedPayload = JSON.parse(atob(paddedPayload))

    if (typeof decodedPayload !== 'object' || !decodedPayload) {
      return null
    }

    const claims = decodedPayload as Record<string, unknown>

    return {
      exp: typeof claims.exp === 'number' ? claims.exp : undefined,
      iss: typeof claims.iss === 'string' ? claims.iss : undefined,
      azp: typeof claims.azp === 'string' ? claims.azp : undefined,
      avatar: typeof claims.avatar === 'string' ? claims.avatar : undefined,
      preferred_username:
        typeof claims.preferred_username === 'string' ? claims.preferred_username : undefined,
      email: typeof claims.email === 'string' ? claims.email : undefined,
      name: typeof claims.name === 'string' ? claims.name : undefined,
      organizations: (() => {
        const raw = claims.organizations
        if (typeof raw !== 'object' || raw === null || Array.isArray(raw)) return undefined
        const validated: Record<string, OrganizationClaim> = {}
        for (const [alias, entry] of Object.entries(raw as Record<string, unknown>)) {
          if (
            typeof entry === 'object' &&
            entry !== null &&
            typeof (entry as OrganizationClaim).id === 'string'
          ) {
            validated[alias] = entry as OrganizationClaim
          }
        }
        return validated
      })(),
    }
  } catch {
    return null
  }
}

/**
 * The realm that issued a session, from its access token, falling back to
 * `fallback` when there is none (no session yet, or a malformed token). The
 * refresh token is opaque and carries no claims.
 */
function sessionRealm(accessToken: string | null, fallback: string): string {
  return realmFromIssuer(accessToken ? decodeJwt(accessToken)?.iss : null) ?? fallback
}

export function useAuth() {
  const navigate = useNavigate()
  const { realm_name = 'master' } = useParams<RouterParams>()
  //const { setAuthTokens, setAuthenticated, setLoading, access_token, refresh_token, expiration, isAuthenticated, isLoading } = userStore()
  const {
    accessToken,
    refreshToken,
    idToken,
    sessionRealm: storedSessionRealm,
    setTokens,
    setSessionRealm,
  } = authStore()
  // The realm that issued the session. It differs from `realm_name` while an
  // admin signed in to one realm (usually master) browses another, and every
  // token-endpoint call and sign-in redirect belongs to it. The stored copy
  // answers once the tokens are gone.
  const authRealm = sessionRealm(accessToken, storedSessionRealm ?? realm_name)
  const {
    expiration,
    isAuthenticated,
    isLoading,
    user,
    setAuthenticated,
    setLoading,
    setUser,
    setExpiration,
  } = userStore()
  const { mutateAsync: exchangeToken } = useTokenMutation()
  const { mutateAsync: revokeToken } = useRevokeTokenMutation()
  const { mutateAsync: remoteLogout } = useLogoutMutation()
  const [hasHydrated, setHasHydrated] = useState<boolean>(
    authStore.persist?.hasHydrated?.() ?? true
  )

  const setAuthTokensWrapper = useCallback(
    (
      access_token: string,
      refresh_token: string | null,
      id_token: string | null = null,
      authenticated: boolean = true
    ) => {
      const decoded = decodeJwt(access_token)
      const expiration = decoded?.exp ? decoded.exp * 1000 : null

      const realm = sessionRealm(access_token, realm_name)
      authRefreshController.reset(realm)
      setSessionRealm(realm)
      setTokens(access_token, refresh_token, id_token)
      setExpiration(expiration)

      if (authenticated) {
        setAuthenticated(true)
      }
    },
    [realm_name, setAuthenticated, setTokens, setSessionRealm, setExpiration]
  )

  const clearAuthState = useCallback(
    (redirectToLogin: boolean, errorMessage?: string) => {
      authRefreshController.reset(authRealm)
      authStore.persist?.clearStorage?.()
      setTokens(null, null, null)
      setUser(null)
      setExpiration(null)
      setAuthenticated(false)
      setLoading(false)

      if (redirectToLogin) {
        const params = new URLSearchParams()
        if (errorMessage) {
          params.set('login_error', errorMessage)
        }

        const suffix = params.toString()
        navigate(`/realms/${authRealm}/authentication/login${suffix ? `?${suffix}` : ''}`, {
          replace: true,
        })
      }
    },
    [navigate, authRealm, setAuthenticated, setExpiration, setLoading, setTokens, setUser]
  )

  function isTokenExpired(): boolean {
    if (!expiration) return true
    return Date.now() > expiration - 60000
  }

  async function logout() {
    try {
      const revokeRequests: Promise<unknown>[] = []
      const accessTokenClaims = accessToken ? decodeJwt(accessToken) : null
      const clientIdFromToken =
        (accessTokenClaims?.azp as string | undefined) || 'security-admin-console'

      if (accessToken) {
        revokeRequests.push(
          revokeToken({
            realm: authRealm,
            data: {
              token: accessToken,
              client_id: clientIdFromToken,
              token_type_hint: 'access_token',
            },
          })
        )
      }

      if (refreshToken) {
        revokeRequests.push(
          revokeToken({
            realm: authRealm,
            data: {
              token: refreshToken,
              client_id: clientIdFromToken,
              token_type_hint: 'refresh_token',
            },
          })
        )
      }

      if (revokeRequests.length > 0) {
        await Promise.allSettled(revokeRequests)
      }

      await remoteLogout({
        realm: authRealm,
        data: {
          id_token_hint: idToken ?? undefined,
          client_id: clientIdFromToken,
        },
      })
    } catch (error) {
      console.error('Failed to clear server-side session cookies during logout:', error)
    } finally {
      clearAuthState(true)
    }
  }

  const refreshAccessToken = useCallback(() => {
    if (!refreshToken) {
      setAuthenticated(false)
      return Promise.resolve()
    }

    return authRefreshController
      .run(authRealm, async () => {
        const response = await exchangeToken({
          realm: authRealm,
          data: {
            grant_type: GrantType.RefreshToken,
            client_id: 'security-admin-console',
            refresh_token: refreshToken,
          },
        })

        setAuthTokensWrapper(
          response.access_token,
          response.refresh_token ?? null,
          response.id_token ?? idToken
        )
      })
      .catch((error: unknown) => {
        const message =
          error instanceof Error && error.message !== 'refresh temporarily blocked'
            ? translate('common:session.expired')
            : undefined

        clearAuthState(Boolean(message), message)
      })
  }, [
    clearAuthState,
    exchangeToken,
    idToken,
    authRealm,
    refreshToken,
    setAuthTokensWrapper,
    setAuthenticated,
  ])

  // A session signed in before the realm was stored still names it in its
  // tokens: keep the stored copy in step, so it is there when they expire.
  useEffect(() => {
    const realm = accessToken ? realmFromIssuer(decodeJwt(accessToken)?.iss) : null
    if (realm && realm !== storedSessionRealm) setSessionRealm(realm)
  }, [accessToken, storedSessionRealm, setSessionRealm])

  useEffect(() => {
    if (!authStore.persist?.onFinishHydration) return
    const unsubscribe = authStore.persist.onFinishHydration(() => {
      setHasHydrated(true)
    })
    return unsubscribe
  }, [])

  useEffect(() => {
    const interval = setInterval(() => {
      if (!isAuthenticated || !accessToken) return
      const payload = decodeJwt(accessToken)

      if (!payload) {
        console.error('Invalid token format')
        return
      }

      const exp = payload.exp
      if (typeof exp !== 'number') {
        console.error('Missing exp claim in token')
        return
      }
      const currentTime = Math.floor(Date.now() / 1000)
      const timeToExpiry = exp - currentTime

      if (timeToExpiry <= 5) {
        void refreshAccessToken()
      }
    }, 1000 * 5)

    return () => clearInterval(interval)
  }, [isAuthenticated, accessToken, refreshAccessToken])

  useEffect(() => {
    if (!hasHydrated) return

    if (!accessToken) {
      setAuthenticated(false)
      setLoading(false)
      return
    }

    const decoded = decodeJwt(accessToken)
    if (!decoded || typeof decoded.exp !== 'number') {
      setAuthenticated(false)
      setLoading(false)
      return
    }

    const user: IUser = {
      avatar: decoded.avatar ?? '',
      preferred_username: decoded.preferred_username ?? '',
      email: decoded.email ?? '',
      name: decoded.name ?? '',
      organizations: decoded.organizations ?? {},
    }

    setUser(user)

    const expTime = decoded.exp * 1000
    const currentTime = Date.now()

    setAuthenticated(expTime > currentTime)
    setLoading(false)
  }, [accessToken, hasHydrated, setAuthenticated, setLoading, setUser])

  return {
    setAuthToken: (value: string) => setAuthTokensWrapper(value, '', null, false),
    setAuthTokens: setAuthTokensWrapper,
    isTokenExpired,
    isAuthenticated,
    isLoading,
    user,
    refreshAccessToken,
    logout,
    clearAuthState,
  }
}
