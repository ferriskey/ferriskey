import { GrantType } from '@/api/api.interface'
import { useTokenMutation } from '@/api/auth.api'
import { userStore } from '@/store/user.store'
import { useEffect } from 'react'
import { useNavigate } from 'react-router'

function decodeJwt(token: string): Record<string, never> | null {
  try {
    const payload = token.split('.')[1]
    return JSON.parse(atob(payload))
  } catch (e) {
    return null
  }
}

export function useAuth() {
  const navigate = useNavigate()
  const { setAuthTokens, setAuthenticated, setLoading, access_token, refresh_token, expiration, isAuthenticated, isLoading } = userStore()
  const { mutate: exchangeToken, data: responseExchangeToken } = useTokenMutation()

  function setAuthTokensWrapper(access_token: string, refresh_token: string) {
    const decoded = decodeJwt(access_token)
    const expiration = decoded?.exp ? decoded.exp * 1000 : null

    if (setAuthTokens) {
      setAuthTokens(access_token, refresh_token, expiration)
    }

    setAuthenticated(true)
  }

  function isTokenExpired(): boolean {
    if (!expiration) return true
    return Date.now() > expiration - 60000
  }

  async function refreshAccessToken() {
    if (!refresh_token) {
      setAuthenticated(false)
      return
    }

    exchangeToken({
      realm: 'master',
      data: {
        grant_type: GrantType.RefreshToken,
        client_id: 'security-admin-console',
        refresh_token: refresh_token,
      }
    })
  }

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
  }, [])

  useEffect(() => {
    if (responseExchangeToken?.access_token) {
      setAuthTokensWrapper(
        responseExchangeToken.access_token,
        responseExchangeToken.refresh_token
      )
    }
  }, [responseExchangeToken])

  useEffect(() => {
    const interval = setInterval(() => {
      if (!isAuthenticated || !access_token) return
      const payload = decodeJwt(access_token)

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
  }, [isAuthenticated, access_token])

  useEffect(() => {
    if (!isLoading) return

    if (!access_token) {
      setAuthenticated(false)
      setLoading(false)
      return
    }

    const decoded = decodeJwt(access_token)
    if (!decoded || !decoded.exp) {
      setAuthenticated(false)
      setLoading(false)
      return
    }

    const expTime = decoded.exp * 1000
    const currentTime = Date.now()

    setAuthenticated(expTime > currentTime)
    setLoading(false)
  }, [])

  return {
    setAuthToken: (value: string) => setAuthTokensWrapper(value, ''),
    setAuthTokens: setAuthTokensWrapper,
    isTokenExpired,
    isAuthenticated,
    isLoading,
    refreshAccessToken,
  }
}
