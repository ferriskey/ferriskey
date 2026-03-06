import { useAuthenticateMutation } from '@/api/auth.api'
import { zodResolver } from '@hookform/resolvers/zod'
import { useCallback, useEffect, useMemo } from 'react'
import { useForm } from 'react-hook-form'
import { useLocation, useNavigate, useParams } from 'react-router'
import { z } from 'zod'
import PageLogin from '../ui/page-login'
import { toast } from 'sonner'
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

  const {
    mutate: authenticate,
    data: authenticateData,
    status: authenticateStatus,
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
    if (!isAuthInitiated) {
      const { query, realm } = getOAuthParams()
      window.location.href = `${window.apiUrl}/realms/${realm}/protocol/openid-connect/auth?${query}`
    }
  }, [isAuthInitiated, getOAuthParams])

  useEffect(() => {
    if (authenticateStatus === 'error') {
      toast.error('Authentication failed. Please check your credentials and try again.')
    }
  }, [authenticateStatus, form])

  const isRedirecting = !isAuthInitiated

  if (isRedirecting) {
    return <PageLogin form={form} onSubmit={onSubmit} isLoading loginSettings={loginSettings} />
  }

  if (!loginSettings) return null

  return (
    <PageLogin
      form={form}
      onSubmit={onSubmit}
      isError={undefined}
      loginSettings={loginSettings}
    />
  )
}
