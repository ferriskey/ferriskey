import { useAuthenticateMutation } from '@/api/auth.api'
import { zodResolver } from '@hookform/resolvers/zod'
import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { useForm } from 'react-hook-form'
import { useLocation, useNavigate, useParams } from 'react-router'
import { z } from 'zod'
import { toast } from 'sonner'
import FloatingActionBar from '@/components/ui/floating-action-bar'
import PageLogin from '../ui/page-login'
import { AuthenticationStatus } from '@/api/api.interface.ts'
import { useGetLoginSettings } from '@/api/realm.api'

const authenticateSchema = z.object({
  username: z.string().min(1, { message: 'Username is required' }),
  password: z.string().min(1, { message: 'Password is required' }),
})

export type AuthenticateSchema = z.infer<typeof authenticateSchema>

export default function PageLoginFeature() {
  const { realm_name } = useParams()
  const navigate = useNavigate()
  const location = useLocation()
  const searchParams = useMemo(() => new URLSearchParams(location.search), [location.search])
  const clientId = searchParams.get('client_id')
  const redirectUri = searchParams.get('redirect_uri')
  const isAuthInitiated = Boolean(clientId && redirectUri)

  const { data: loginSettings } = useGetLoginSettings({ realm: realm_name })

  const [showSessionBar, setShowSessionBar] = useState(false)
  const [countdown, setCountdown] = useState<number | null>(null)
  const timerRef = useRef<number | null>(null)
  const countdownRef = useRef<number | null>(null)
  const autoRefreshRef = useRef<number | null>(null)
  const restartAuthFlowRef = useRef<() => void>(() => { })

  const getAuthParamsFromUrl = useCallback(() => {
    return {
      clientId: clientId ?? 'security-admin-console',
      redirectUri:
        redirectUri ??
        `${window.location.origin}/realms/${realm_name ?? 'master'}/authentication/callback`,
    }
  }, [clientId, redirectUri, realm_name])

  const getOAuthParams = useCallback(() => {
    const state = crypto.randomUUID()
    sessionStorage.setItem('oauth_state', state)
    const { clientId, redirectUri } = getAuthParamsFromUrl()

    return {
      query: new URLSearchParams({
        response_type: 'code',
        client_id: clientId,
        redirect_uri: redirectUri,
        scope: 'openid profile email',
        state,
      }).toString(),
      realm: realm_name ?? 'master',
    }
  }, [getAuthParamsFromUrl, realm_name])

  const {
    mutate: authenticate,
    data: authenticateData,
    status: authenticateStatus,
    error: authenticateError,
    reset: resetAuthenticate,
  } = useAuthenticateMutation()

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
      const newState = sessionStorage.getItem('oauth_state') ?? crypto.randomUUID()

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

  const loginError = searchParams.get('login_error')

  const form = useForm<AuthenticateSchema>({
    resolver: zodResolver(authenticateSchema),
    defaultValues: {
      username: '',
      password: '',
    },
  })

  useEffect(() => {
    if (!authenticateData) return
    if (authenticateData.url) {
      window.location.href = authenticateData.url
    }

    if (
      authenticateData.status === AuthenticationStatus.RequiresActions &&
      authenticateData.required_actions &&
      authenticateData.required_actions.length > 0 &&
      authenticateData.token
    ) {
      const firstRequiredAction = authenticateData.required_actions[0]

      navigate(
        `/realms/${realm_name}/authentication/required-action?execution=${firstRequiredAction.toUpperCase()}&client_data=${authenticateData.token}`
      )
    }

    if (authenticateData.status === AuthenticationStatus.RequiresOtpChallenge) {
      navigate(`/realms/${realm_name}/authentication/otp?token=${authenticateData.token}`)
    }
  }, [authenticateData, form, navigate, realm_name])

  function onSubmit(data: AuthenticateSchema) {
    const { clientId } = getAuthParamsFromUrl()
    authenticate({
      data,
      realm: realm_name ?? 'master',
      clientId,
    })
  }

  useEffect(() => {
    if (!isAuthInitiated && !loginError) {
      const { query, realm } = getOAuthParams()
      window.location.href = `${window.apiUrl}/realms/${realm}/protocol/openid-connect/auth?${query}`
    }
  }, [isAuthInitiated, getOAuthParams, loginError])

  const authErrorStatus = (authenticateError as { status?: number } | null)?.status

  const authErrorMessage =
    authenticateStatus === 'error'
      ? (authenticateError?.message ??
        'Authentication failed. Please check your credentials and try again.')
      : null

  const errorMessage = loginError ?? authErrorMessage

  const isSessionError =
    (errorMessage &&
      /(session|expired|invalid[_-]?session|session[_-]?not[_-]?found)/i.test(errorMessage)) ||
    authErrorStatus === 500

  const showFloatingActionBar = isSessionError || showSessionBar

  const isRedirecting = !isAuthInitiated && !loginError

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

  if (isRedirecting) {
    return <PageLogin form={form} onSubmit={onSubmit} isLoading loginSettings={loginSettings} />
  }

  if (!loginSettings) return null

  return (
    <>
      <PageLogin
        form={form}
        onSubmit={onSubmit}
        isError={undefined}
        loginSettings={loginSettings}
        errorMessage={errorMessage}
      />
      <FloatingActionBar
        show={showFloatingActionBar}
        title='Session expired'
        description={
          countdown !== null
            ? `Refreshing automatically in ${countdown}s...`
            : 'Restart your session to continue.'
        }
        onCancel={countdown !== null ? cancelAutoRefresh : undefined}
        actions={[
          { label: 'Refresh session', variant: 'default', onClick: () => restartAuthFlow() },
        ]}
      />
    </>
  )
}
