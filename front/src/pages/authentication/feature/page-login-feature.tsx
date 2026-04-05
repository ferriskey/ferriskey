import { useAuthenticateMutation } from '@/api/auth.api'
import { zodResolver } from '@hookform/resolvers/zod'
import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { useForm } from 'react-hook-form'
import { useLocation, useNavigate, useParams } from 'react-router'
import { z } from 'zod'
import { toast } from 'sonner'
import FloatingActionBar from '@/components/ui/floating-action-bar'
import PageLogin, { LoginErrorPage, MagicLinkStep } from '../ui/page-login'
import { AuthenticationStatus } from '@/api/api.interface.ts'
import { useGetLoginSettings } from '@/api/realm.api'
import { usePasskeyRequestOptionsMutation, usePasskeyAuthenticateMutation } from '@/api/passkey.api'
import { useSendMagicLink } from '@/api/trident.api'
import { isWebAuthnAvailable, isConditionalMediationAvailable, startAuthentication, startConditionalAuthentication } from '@/lib/webauthn'
import { magicLinkSchema, MagicLinkSchema } from '@/pages/authentication/schemas/magic-link.schema'

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

  // Detect stale realm in redirect_uri.
  // When the user edits the realm in the URL (e.g. /realms/master → /realms/cloud-iam)
  // the old redirect_uri (".../realms/master/authentication/callback") stays in the query
  // params. If we don't catch this, the OAuth session is created with the wrong callback
  // and the user ends up redirected to the wrong realm after login.
  const currentRealm = realm_name ?? 'master'
  const realmCallbackUri = `${window.location.origin}/realms/${currentRealm}/authentication/callback`
  const isRedirectUriStale =
    redirectUri !== null &&
    redirectUri.includes('/realms/') &&
    !redirectUri.includes(`/realms/${currentRealm}/`)

  // Auth is "initiated" (OAuth session exists) only when both params are present
  // AND the redirect_uri belongs to the current realm.
  const isAuthInitiated = Boolean(clientId && redirectUri) && !isRedirectUriStale

  const { data: loginSettings } = useGetLoginSettings({ realm: realm_name })

  const [showSessionBar, setShowSessionBar] = useState(false)
  const [countdown, setCountdown] = useState<number | null>(null)
  const timerRef = useRef<number | null>(null)
  const countdownRef = useRef<number | null>(null)
  const autoRefreshRef = useRef<number | null>(null)
  const restartAuthFlowRef = useRef<() => void>(() => { })

  const getAuthParamsFromUrl = useCallback(() => {
    const resolvedClientId = clientId ?? 'security-admin-console'
    // For the webapp's own OAuth client, always derive redirect_uri from the
    // current realm — never use a stale value carried over from another realm's
    // query params.
    const resolvedRedirectUri =
      resolvedClientId === 'security-admin-console'
        ? realmCallbackUri
        : (redirectUri ?? realmCallbackUri)
    return {
      clientId: resolvedClientId,
      redirectUri: resolvedRedirectUri,
    }
  }, [clientId, redirectUri, realmCallbackUri])

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

  const { mutateAsync: requestPasskeyOptionsAsync, mutate: requestPasskeyOptions } = usePasskeyRequestOptionsMutation()
  const { mutateAsync: authenticatePasskeyAsync, mutate: authenticatePasskey } = usePasskeyAuthenticateMutation()
  const { mutate: sendMagicLink, isPending: isMagicLinkLoading } = useSendMagicLink()
  const [magicLinkStep, setMagicLinkStep] = useState<MagicLinkStep>('idle')
  const [isPasskeyLoading, setIsPasskeyLoading] = useState(false)

  const magicLinkForm = useForm<MagicLinkSchema>({
    resolver: zodResolver(magicLinkSchema),
    defaultValues: { email: '' },
  })
  const [conditionalUIVersion, setConditionalUIVersion] = useState(0)
  const conditionalAbortRef = useRef<AbortController | null>(null)

  // Conditional UI: autofill passkeys in the username field (Apple Passkeys, Chrome, etc.)
  useEffect(() => {
    if (!loginSettings?.passkey_enabled || !isAuthInitiated) return

    let aborted = false
    const abortController = new AbortController()
    conditionalAbortRef.current = abortController

    const startConditionalUI = async () => {
      const available = await isConditionalMediationAvailable()
      if (!available || aborted) return

      try {
        const response = await requestPasskeyOptionsAsync({
          realm: realm_name,
          data: {},
        })

        if (aborted) return

        const assertion = await startConditionalAuthentication(
          response.publicKey,
          abortController.signal
        )

        if (!assertion || aborted) return

        const result = await authenticatePasskeyAsync({
          realm: realm_name,
          data: assertion,
        })

        if (result.login_url) {
          window.location.href = result.login_url
        }
      } catch {
        // Conditional UI was aborted or failed silently — this is expected
      }
    }

    startConditionalUI()

    return () => {
      aborted = true
      abortController.abort()
      conditionalAbortRef.current = null
    }
  }, [loginSettings?.passkey_enabled, isAuthInitiated, realm_name, requestPasskeyOptionsAsync, authenticatePasskeyAsync, conditionalUIVersion])

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

  const onPasskeyLogin = useCallback(() => {
    if (!isWebAuthnAvailable()) {
      toast.error('WebAuthn is not supported in this browser')
      return
    }

    // Abort any ongoing conditional UI request
    conditionalAbortRef.current?.abort()
    conditionalAbortRef.current = null

    setIsPasskeyLoading(true)

    requestPasskeyOptions(
      { realm: realm_name, data: {} },
      {
        onSuccess: async (response) => {
          try {
            const assertion = await startAuthentication(response.publicKey)
            authenticatePasskey(
              { realm: realm_name, data: assertion },
              {
                onSuccess: (result) => {
                  if (result.login_url) {
                    window.location.href = result.login_url
                  }
                },
                onError: () => {
                  toast.error('Passkey authentication failed')
                  setIsPasskeyLoading(false)
                  setConditionalUIVersion(v => v + 1)
                },
              }
            )
          } catch {
            setIsPasskeyLoading(false)
          }
        },
        onError: () => {
          toast.error('Failed to start passkey authentication')
          setIsPasskeyLoading(false)
          setConditionalUIVersion(v => v + 1)
        },
      }
    )
  }, [form, realm_name, requestPasskeyOptions, authenticatePasskey])

  const onMagicLinkLogin = useCallback(() => {
    setMagicLinkStep('form')
  }, [])

  const onMagicLinkBack = useCallback(() => {
    setMagicLinkStep('idle')
    magicLinkForm.reset()
  }, [magicLinkForm])

  const onMagicLinkSubmit = useCallback((data: MagicLinkSchema) => {
    sendMagicLink(
      {
        path: { realm_name: realm_name ?? 'master' },
        body: { email: data.email },
      },
      {
        onSuccess: () => {
          setMagicLinkStep('sent')
        },
        onError: () => {
          toast.error('Failed to send magic link')
        },
      }
    )
  }, [realm_name, sendMagicLink])

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

  // Fatal configuration error (e.g. "Invalid redirect URI", "Client not found").
  // The backend redirected here because it can't trust the redirect_uri.
  // Show a clean error card — do NOT render the login form or retry the OAuth flow.
  if (loginError && !isAuthInitiated) {
    return <LoginErrorPage errorMessage={loginError} />
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
        onPasskeyLogin={loginSettings?.passkey_enabled ? onPasskeyLogin : undefined}
        isPasskeyLoading={isPasskeyLoading}
        onMagicLinkLogin={loginSettings?.magic_link_enabled ? onMagicLinkLogin : undefined}
        isMagicLinkLoading={isMagicLinkLoading}
        magicLinkStep={loginSettings?.magic_link_enabled ? magicLinkStep : undefined}
        magicLinkForm={magicLinkForm}
        onMagicLinkSubmit={onMagicLinkSubmit}
        onMagicLinkBack={onMagicLinkBack}
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
