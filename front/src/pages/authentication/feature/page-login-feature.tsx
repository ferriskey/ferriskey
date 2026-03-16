import { useAuthenticateMutation } from '@/api/auth.api'
import { zodResolver } from '@hookform/resolvers/zod'
import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { useForm } from 'react-hook-form'
import { useLocation, useNavigate, useParams } from 'react-router'
import { z } from 'zod'
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
  const timerRef = useRef<number | null>(null)

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
        redirect_uri: redirectUri, // URL de callback de votre app
        scope: 'openid profile email',
        state,
      }).toString(),
      realm: realm_name ?? 'master',
    }
  }, [getAuthParamsFromUrl, realm_name])

  const restartAuthFlow = useCallback(() => {
    const { query, realm } = getOAuthParams()
    setShowSessionBar(false)
    window.location.href = `${window.apiUrl}/realms/${realm}/protocol/openid-connect/auth?${query}`
  }, [getOAuthParams])

  const scheduleSessionExpirationBar = useCallback(() => {
    if (timerRef.current) {
      window.clearTimeout(timerRef.current)
    }
    timerRef.current = window.setTimeout(() => {
      setShowSessionBar(true)
    }, 300_000)
  }, [])

  const loginError = searchParams.get('login_error')

  const {
    mutate: authenticate,
    data: authenticateData,
    status: authenticateStatus,
    error: authenticateError,
  } = useAuthenticateMutation()

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
      /(session|expired|invalid[_-]?session|session[_-]?not[_-]?found|internal server error)/i.test(
        errorMessage
      )) ||
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
        description='Restart your session to continue.'
        actions={[{ label: 'Refresh session', variant: 'default', onClick: restartAuthFlow }]}
      />
    </>
  )
}
