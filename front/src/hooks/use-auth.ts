import { userStore } from '@/store/user.store'
import { useEffect, useRef } from 'react'
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
  const { setAuthTokens, switchIsAuthenticated, access_token, refresh_token, expiration } = userStore()
  const refreshTimeoutRef = useRef<NodeJS.Timeout | null>(null)

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

  const isTokenExpired = () => {
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

  const setupTokenRefresh = () => {
    // Clear any existing timeout
    if (refreshTimeoutRef.current) {
      clearTimeout(refreshTimeoutRef.current)
      refreshTimeoutRef.current = null
    }

    if (!access_token || !expiration) return

    const timeToExpiry = expiration - Date.now()
    const refreshBuffer = 60000 // 1 minute before expiry

    console.log(`Token expires in ${Math.round(timeToExpiry / 1000)} seconds`)

    // Don't set a timer if the token is already expired or will expire soon
    if (timeToExpiry <= refreshBuffer) {
      console.log('Token is expired or will expire soon, refreshing now')
      refreshAccessToken()
      return
    }

    // Schedule refresh before token expires
    const refreshTime = timeToExpiry - refreshBuffer
    console.log(`Scheduling token refresh in ${Math.round(refreshTime / 1000)} seconds`)

    refreshTimeoutRef.current = setTimeout(async () => {
      const success = await refreshAccessToken()
      if (success) {
        // Token refreshed, no need to set up another timer as the useEffect will trigger
      } else {
        // Failed to refresh, try again in 30 seconds if still authenticated
        if (userStore.getState().isAuthenticated) {
          refreshTimeoutRef.current = setTimeout(() => {
            setupTokenRefresh()
          }, 30000)
        }
      }
    }, refreshTime)
  }

  // Set up token refresh mechanism
  useEffect(() => {
    setupTokenRefresh()

    return () => {
      if (refreshTimeoutRef.current) {
        clearTimeout(refreshTimeoutRef.current)
      }
    }
  }, [access_token, expiration])

  return {
    setAuthToken,
    setAuthTokens: setAuthTokensWrapper,
    isTokenExpired,
    refreshAccessToken,
  }
}
