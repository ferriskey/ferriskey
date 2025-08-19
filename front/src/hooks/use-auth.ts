import { GrantType } from '@/api/core.interface'
import { useTokenMutation } from '@/api/auth.api'
import { RouterParams } from '@/routes/router'
import { authStore } from '@/store/auth.store'
import userStore from '@/store/user.store'
import { useCallback, useEffect } from 'react'
import { useNavigate, useParams } from 'react-router'
import { IUser } from '@/contracts/states.interface'

function decodeJwt(token: string): Record<string, never> | null {
  try {
    const payload = token.split('.')[1]
    return JSON.parse(atob(payload))
  } catch {
    return null
  }
}

export function useAuth() {
  const navigate = useNavigate()
  const { realm_name = 'master' } = useParams<RouterParams>()
  //const { setAuthTokens, setAuthenticated, setLoading, access_token, refresh_token, expiration, isAuthenticated, isLoading } = userStore()
  const { accessToken, refreshToken, setTokens } = authStore()
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
  const { mutate: exchangeToken, data: responseExchangeToken } = useTokenMutation()

  const setAuthTokensWrapper = useCallback(
    (access_token: string, refresh_token: string, authenticated: boolean = true) => {
      const decoded = decodeJwt(access_token)
      const expiration = decoded?.exp ? decoded.exp * 1000 : null

      setTokens(access_token, refresh_token)
      setExpiration(expiration)

      if (authenticated) {
        setAuthenticated(true)
      }
    },
    [setAuthenticated, setTokens, setExpiration]
  )

  function isTokenExpired(): boolean {
    if (!expiration) return true
    return Date.now() > expiration - 60000
  }

  function logout() {
    localStorage.removeItem('auth')
    setAuthenticated(false)
    setLoading(true)
  }

  const refreshAccessToken = useCallback(() => {
    if (!refreshToken) {
      setAuthenticated(false)
      return
    }

    exchangeToken({
      realm: realm_name,
      data: {
        grant_type: GrantType.RefreshToken,
        client_id: 'security-admin-console',
        refresh_token: refreshToken,
      },
    })
  }, [refreshToken, exchangeToken, realm_name, setAuthenticated])

  useEffect(() => {
    const interval = setInterval(() => {
      if (location.pathname.includes('authentication')) return
      const authState = localStorage.getItem('auth')

      if (!authState) {
        setAuthenticated(false)
        localStorage.removeItem('auth')
        navigate('/authentication/login')
      }
    }, 1000)

    return () => clearInterval(interval)
  }, [navigate, setAuthenticated])

  useEffect(() => {
    if (responseExchangeToken?.access_token) {
      setAuthTokensWrapper(responseExchangeToken.access_token, responseExchangeToken.refresh_token)
    }
  }, [responseExchangeToken, setAuthTokensWrapper])

  useEffect(() => {
    const interval = setInterval(() => {
      if (!isAuthenticated || !accessToken) return
      const payload = decodeJwt(accessToken)

      if (!payload) {
        console.error('Invalid token format')
        return
      }

      const exp = payload.exp
      const currentTime = Math.floor(Date.now() / 1000)
      const timeToExpiry = exp - currentTime

      if (timeToExpiry <= 5) {
        refreshAccessToken()
      }
    }, 1000 * 5)

    return () => clearInterval(interval)
  }, [isAuthenticated, accessToken, refreshAccessToken])

  useEffect(() => {
    if (!isLoading) return

    if (!accessToken) {
      setAuthenticated(false)
      setLoading(false)
      return
    }

    const decoded = decodeJwt(accessToken)
    if (!decoded || !decoded.exp) {
      setAuthenticated(false)
      setLoading(false)
      return
    }

    const user: IUser = {
      avatar: decoded.avatar,
      preferred_username: decoded.preferred_username,
      email: decoded.email,
      name: decoded.name,
    }

    setUser(user)

    const expTime = decoded.exp * 1000
    const currentTime = Date.now()

    setAuthenticated(expTime > currentTime)
    setLoading(false)
  }, [accessToken, isLoading, setAuthenticated, setLoading, setUser])

  return {
    setAuthToken: (value: string) => setAuthTokensWrapper(value, '', false),
    setAuthTokens: setAuthTokensWrapper,
    isTokenExpired,
    isAuthenticated,
    isLoading,
    user,
    refreshAccessToken,
    logout,
  }
}
