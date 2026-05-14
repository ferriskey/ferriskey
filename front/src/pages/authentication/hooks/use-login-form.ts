import { zodResolver } from '@hookform/resolvers/zod'
import { useCallback, useEffect } from 'react'
import { useForm } from 'react-hook-form'
import { useNavigate } from 'react-router'
import { z } from 'zod'
import { AuthenticationStatus } from '@/api/api.interface.ts'
import { useAuthenticateMutation } from '@/api/auth.api'

export const authenticateSchema = z.object({
  username: z.string().min(1, { message: 'Username is required' }),
  password: z.string().min(1, { message: 'Password is required' }),
})

export type AuthenticateSchema = z.infer<typeof authenticateSchema>

type Options = {
  realm_name: string | undefined
  loginError: string | null
  getAuthParamsFromUrl: () => { clientId: string; redirectUri: string }
}

export function useLoginForm({ realm_name, loginError, getAuthParamsFromUrl }: Options) {
  const navigate = useNavigate()

  const {
    mutate: authenticate,
    data: authenticateData,
    status: authenticateStatus,
    error: authenticateError,
    reset: resetAuthenticate,
  } = useAuthenticateMutation()

  const form = useForm<AuthenticateSchema>({
    resolver: zodResolver(authenticateSchema),
    defaultValues: { username: '', password: '' },
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
  }, [authenticateData, navigate, realm_name])

  const onSubmit = useCallback(
    (data: AuthenticateSchema) => {
      const { clientId } = getAuthParamsFromUrl()
      authenticate({
        data,
        realm: realm_name ?? 'master',
        clientId,
      })
    },
    [authenticate, getAuthParamsFromUrl, realm_name]
  )

  const authErrorStatus = (authenticateError as { status?: number } | null)?.status

  const authErrorMessage =
    authenticateStatus === 'error'
      ? (authenticateError?.message ??
        'Authentication failed. Please check your credentials and try again.')
      : null

  const errorMessage = loginError ?? authErrorMessage

  const isSessionError = Boolean(
    (errorMessage &&
      /(session|expired|invalid[_-]?session|session[_-]?not[_-]?found)/i.test(errorMessage)) ||
      authErrorStatus === 500
  )

  return {
    form,
    onSubmit,
    errorMessage,
    isSessionError,
    resetAuthenticate,
  }
}
