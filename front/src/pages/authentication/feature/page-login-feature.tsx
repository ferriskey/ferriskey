import { useAuthenticateMutation } from '@/api/auth.api'
import { zodResolver } from '@hookform/resolvers/zod'
import { useCallback, useEffect, useState } from 'react'
import { useForm } from 'react-hook-form'
import { useNavigate, useParams } from 'react-router'
import { z } from 'zod'
import PageLogin from '../ui/page-login'
import { toast } from 'sonner'
import { AuthenticationStatus } from '@/api/api.interface.ts'

const authenticateSchema = z.object({
  username: z.string().min(1),
  password: z.string().min(1),
})

export type AuthenticateSchema = z.infer<typeof authenticateSchema>

export default function PageLoginFeature() {
  const { realm_name } = useParams()
  const [isAuthInitiated, setIsAuthInitiated] = useState<boolean>(false)
  const [isSetup, setIsSetup] = useState(false)
  const navigate = useNavigate()


  const getOAuthParams = useCallback(() => {
    const state = crypto.randomUUID()
    sessionStorage.setItem('oauth_state', state)

    return {
      query: new URLSearchParams({
        response_type: 'code',
        client_id: 'security-admin-console',
        redirect_uri: `${window.location.origin}/realms/${realm_name ?? 'master'}/authentication/callback`, // URL de callback de votre app
        scope: 'openid profile email',
        state,
      }).toString(),
      realm: realm_name ?? 'master',
    }
  }, [realm_name])

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
    const cookies = document.cookie.split(';').reduce(
      (acc, cookie) => {
        const [key, value] = cookie.trim().split('=')
        acc[key] = value
        return acc
      },
      {} as Record<string, string>
    )

    const sessionCode = cookies['FERRISKEY_SESSION'] || '123456' // Fallback to default if not found
    authenticate({
      data,
      realm: realm_name ?? 'master',
      clientId: 'security-admin-console',
      sessionCode,
    })
  }

  useEffect(() => {
    const urlParams = new URLSearchParams(window.location.search)
    const clientId = urlParams.get('client_id')
    const redirectUri = urlParams.get('redirect_uri')

    if (clientId && redirectUri) {
      setIsAuthInitiated(true)
    }

    setIsSetup(true)
  }, [])

  useEffect(() => {
    if (isSetup && !isAuthInitiated) {
      const { query, realm } = getOAuthParams()
      window.location.href = `${window.apiUrl}/realms/${realm}/protocol/openid-connect/auth?${query}`
    }
  }, [isSetup, isAuthInitiated, getOAuthParams])

  useEffect(() => {
    if (authenticateStatus === 'error') {
      toast.error('Authentication failed. Please check your credentials and try again.')
      form.reset()
    }
  }, [authenticateStatus, form])

  return <PageLogin form={form} onSubmit={onSubmit} isError={undefined} />
}
