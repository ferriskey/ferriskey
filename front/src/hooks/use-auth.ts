import { userStore } from '@/store/user.store'
import { useEffect } from 'react'
import { GrantType } from '@/api/api.interface'
import axios from 'axios'

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
  const { setAuthTokens, switchIsAuthenticated, access_token, refresh_token, expiration, isAuthenticated, isLoading } = userStore()

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
    try {
      if (!refresh_token) {
        console.error('No refresh token available')
        switchIsAuthenticated(false)
        return false
      }

      console.log('Refreshing access token...')

      const response = await axios.post('/realms/master/protocol/openid-connect/token', {
        grant_type: GrantType.RefreshToken,
        client_id: 'security-admin-console',
        refresh_token: refresh_token,
      })

      if (response.data.access_token) {
        console.log('Token refreshed successfully')
        setAuthTokensWrapper(response.data.access_token, response.data.refresh_token || refresh_token)
        return true
      }
      return false
    } catch (error) {
      console.error('Error refreshing token:', error)
      switchIsAuthenticated(false)
      return false
    }
  }

  useEffect(() => {
    const interval = setInterval(() => {
      if (!isAuthenticated || !access_token) {
        console.log('User is not authenticated or no access token available')
        return
      }

      const payload = decodeJwt(access_token)

      if (!payload) {
        console.error('Invalid token format')
        return
      }

      const exp = payload.exp * 1000
      const currentTime = Date.now()

      const timeToExpiry = exp - currentTime
      console.log(`Token expires in ${Math.round(timeToExpiry / 1000)} seconds`)

      console.log(exp, currentTime)

      if (timeToExpiry / 1000 <= 5) {
        console.log('Token expired, refreshing...')
        refreshAccessToken()
      }
    }, 1000 * 5)

    return () => {
      clearInterval(interval)
    }
  }, [isAuthenticated, access_token])

  return {
    setAuthToken,
    setAuthTokens: setAuthTokensWrapper,
    isTokenExpired,
    isAuthenticated,
    isLoading,
    refreshAccessToken,
  }
}
