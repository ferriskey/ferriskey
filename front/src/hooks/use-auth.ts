import { userStore } from '@/store/user.store'
import { useEffect } from 'react'
import { GrantType } from '@/api/api.interface'
import { useTokenMutation } from '@/api/auth.api'

function decodeJwt(token: string) {
  try {
    const payload = token.split('.')[1]
    const decoded = JSON.parse(atob(payload))
    return decoded
  } catch (e) {
    return null
  }
}

export function useAuth() {
  const { setAuthTokens, switchIsAuthenticated, switchIsLoading, access_token, refresh_token, expiration, isAuthenticated, isLoading } = userStore()
  const { mutate: exchangeToken, data: responseExchangeToken } = useTokenMutation()

  const setAuthTokensWrapper = (access_token: string, refresh_token: string) => {
    const decoded = decodeJwt(access_token)
    const expiration = decoded?.exp ? decoded.exp * 1000 : null
    if (setAuthTokens) {
      setAuthTokens(access_token, refresh_token, expiration)
    }
    switchIsAuthenticated(true)
  }

  const setAuthToken = (access_token: string) => {
    setAuthTokensWrapper(access_token, '')
  }

  const isTokenExpired = (): boolean => {
    if (!expiration) return true
    // Check if token is expired or will expire in the next 60 seconds
    return Date.now() > expiration - 60000
  }

  const refreshAccessToken = async () => {
    if (!refresh_token) {
      switchIsAuthenticated(false)
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
    if (!responseExchangeToken) return

    if (responseExchangeToken.access_token) {
      setAuthTokensWrapper(responseExchangeToken.access_token, responseExchangeToken.refresh_token)
    }
  }, [responseExchangeToken])

  useEffect(() => {
    const interval = setInterval(() => {
      if (!isAuthenticated || !access_token) return

      console.log('Checking token expiration...')
      ;
      

      const payload = decodeJwt(access_token)

      if (!payload) {
        console.error('Invalid token format')
        return
      }

      const exp = payload.exp
      const currentTime = Math.floor(Date.now() / 1000)

      console.log(`exp: ${exp}, currentTime: ${currentTime}`);
      

      const timeToExpiry = exp - currentTime

      console.log('Time to expiry:', timeToExpiry)

      if (timeToExpiry <= 5) {
        refreshAccessToken()
      }
    }, 1000 * 5)

    return () => {
      clearInterval(interval)
    }
  }, [isAuthenticated, access_token])

  useEffect(() => {
    if (!isLoading) return

    if (!access_token) {
      switchIsAuthenticated(false)
      switchIsLoading(false)
      return
    }

    const decoded = decodeJwt(access_token)
    if (!decoded || !decoded.exp) {
      switchIsAuthenticated(false)
      switchIsLoading(false)
      return
    }

    const expTime = decoded.exp * 1000
    const currentTime = Date.now()

    if (expTime > currentTime) {
      switchIsAuthenticated(true)
      switchIsLoading(false)
    } else {
      switchIsAuthenticated(false)
      switchIsLoading(false)
    }
  }, [])

  return {
    setAuthToken,
    setAuthTokens: setAuthTokensWrapper,
    isTokenExpired,
    isAuthenticated,
    isLoading,
    refreshAccessToken,
  }
}
