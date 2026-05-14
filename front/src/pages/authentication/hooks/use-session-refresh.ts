import { useCallback, useEffect, useRef, useState } from 'react'
import { useNavigate } from 'react-router'
import { toast } from 'sonner'

type Options = {
  isRedirecting: boolean
  isSessionError: boolean
  getOAuthParams: () => { query: string; realm: string }
  getAuthParamsFromUrl: () => { clientId: string; redirectUri: string }
  resetAuthenticate: () => void
}

export function useSessionRefresh({
  isRedirecting,
  isSessionError,
  getOAuthParams,
  getAuthParamsFromUrl,
  resetAuthenticate,
}: Options) {
  const navigate = useNavigate()
  const [showSessionBar, setShowSessionBar] = useState(false)
  const [countdown, setCountdown] = useState<number | null>(null)
  const timerRef = useRef<number | null>(null)
  const countdownRef = useRef<number | null>(null)
  const autoRefreshRef = useRef<number | null>(null)
  const restartAuthFlowRef = useRef<() => void>(() => {})

  const scheduleSessionExpirationBar = useCallback(() => {
    if (timerRef.current) {
      window.clearTimeout(timerRef.current)
    }
    timerRef.current = window.setTimeout(() => {
      setShowSessionBar(true)
    }, 600_000)
  }, [])

  const clearAutoRefreshTimers = useCallback(() => {
    if (countdownRef.current) window.clearInterval(countdownRef.current)
    if (autoRefreshRef.current) window.clearTimeout(autoRefreshRef.current)
    countdownRef.current = null
    autoRefreshRef.current = null
  }, [])

  const cancelAutoRefresh = useCallback(() => {
    clearAutoRefreshTimers()
    setCountdown(null)
  }, [clearAutoRefreshTimers])

  const restartAuthFlow = useCallback(async () => {
    cancelAutoRefresh()

    const { query, realm } = getOAuthParams()

    await fetch(`${window.apiUrl}/realms/${realm}/protocol/openid-connect/auth?${query}`, {
      credentials: 'include',
      redirect: 'manual',
    })

    try {
      resetAuthenticate()

      const { clientId: cId, redirectUri: rUri } = getAuthParamsFromUrl()
      const newState = new URLSearchParams(query).get('state') ?? crypto.randomUUID()

      navigate(
        `/realms/${realm}/authentication/login?client_id=${cId}&redirect_uri=${rUri}&state=${newState}`,
        { replace: true }
      )

      setShowSessionBar(false)
      scheduleSessionExpirationBar()
      toast.success('Session refreshed', { description: 'You can now log in again.' })
    } catch {
      toast.error('Session refresh failed', { description: 'Please try again.' })
    }
  }, [
    cancelAutoRefresh,
    getOAuthParams,
    getAuthParamsFromUrl,
    navigate,
    scheduleSessionExpirationBar,
    resetAuthenticate,
  ])

  useEffect(() => {
    restartAuthFlowRef.current = restartAuthFlow
  }, [restartAuthFlow])

  const showFloatingActionBar = isSessionError || showSessionBar

  useEffect(() => {
    if (isRedirecting) return

    if (timerRef.current) {
      window.clearTimeout(timerRef.current)
    }

    if (!isSessionError) {
      scheduleSessionExpirationBar()
    }

    return () => {
      if (timerRef.current) {
        window.clearTimeout(timerRef.current)
      }
    }
  }, [isRedirecting, scheduleSessionExpirationBar, isSessionError])

  useEffect(() => {
    if (!showFloatingActionBar) {
      clearAutoRefreshTimers()
      return
    }

    const initId = window.setTimeout(() => setCountdown(5), 0)

    countdownRef.current = window.setInterval(() => {
      setCountdown((prev) => (prev !== null && prev > 1 ? prev - 1 : prev))
    }, 1000)

    autoRefreshRef.current = window.setTimeout(() => {
      restartAuthFlowRef.current()
    }, 5000)

    return () => {
      clearAutoRefreshTimers()
      window.clearTimeout(initId)
    }
  }, [showFloatingActionBar, clearAutoRefreshTimers])

  return {
    showFloatingActionBar,
    countdown,
    cancelAutoRefresh,
    restartAuthFlow,
  }
}
