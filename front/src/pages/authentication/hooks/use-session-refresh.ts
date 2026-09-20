import { useCallback, useEffect, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'
import { AUTH_NAMESPACE } from '../constants'

type Options = {
  isRedirecting: boolean
  isSessionError: boolean
  getOAuthParams: () => Promise<{ query: string; realm: string }>
}

export function useSessionRefresh({ isRedirecting, isSessionError, getOAuthParams }: Options) {
  const { t } = useTranslation(AUTH_NAMESPACE)
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

    try {
      const { query, realm } = await getOAuthParams()

      window.location.href = `${window.apiUrl}/realms/${realm}/protocol/openid-connect/auth?${query}`
    } catch {
      toast.error(t('session.refresh_failed'), {
        description: t('session.refresh_failed_description'),
      })
    }
  }, [cancelAutoRefresh, getOAuthParams, t])

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
