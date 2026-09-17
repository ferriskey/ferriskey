import { zodResolver } from '@hookform/resolvers/zod'
import { useCallback, useEffect } from 'react'
import { useForm } from 'react-hook-form'
import { useTranslation } from 'react-i18next'
import { useNavigate } from 'react-router'
import { z } from 'zod'
import { AuthenticationStatus } from '@/api/api.interface.ts'
import { useAuthenticateMutation } from '@/api/auth.api'
import { apiErrorMessage, apiErrorReason } from '@/lib/api-error'
import { translate } from '@/lib/i18n'
import { AUTH_NAMESPACE } from '../constants'

const MAINTENANCE_REASON = 'client_under_maintenance'

const SESSION_ERROR_REASONS = new Set([
  'unauthorized',
  'session_expired',
  'session_not_found',
  'session_revoked',
  'session_create_error',
  'session_delete_error',
  'invalid_session',
  'expired_token',
])

export const authenticateSchema = z.object({
  username: z
    .string()
    .min(1, { error: () => translate(`${AUTH_NAMESPACE}:validation.username_required`) }),
  password: z
    .string()
    .min(1, { error: () => translate(`${AUTH_NAMESPACE}:validation.password_required`) }),
})

export type AuthenticateSchema = z.infer<typeof authenticateSchema>

type Options = {
  realm_name: string | undefined
  loginError: string | null
  getAuthParamsFromUrl: () => { clientId: string; redirectUri: string }
}

export function useLoginForm({ realm_name, loginError, getAuthParamsFromUrl }: Options) {
  const navigate = useNavigate()
  const { t } = useTranslation(AUTH_NAMESPACE)

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
      authenticateData.required_actions.length > 0
    ) {
      const firstRequiredAction = authenticateData.required_actions[0]

      navigate(
        `/realms/${realm_name}/authentication/required-action?execution=${firstRequiredAction.toUpperCase()}`
      )
    }

    if (authenticateData.status === AuthenticationStatus.RequiresOtpChallenge) {
      navigate(`/realms/${realm_name}/authentication/otp`, {
        state: { email: authenticateData.email ?? null },
      })
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
  const authErrorReason = apiErrorReason(authenticateError)

  const authErrorMessage =
    authenticateStatus === 'error'
      ? apiErrorMessage(authenticateError, t('login.failed'))
      : null

  const errorMessage = loginError ?? authErrorMessage

  const isSessionError = Boolean(
    (authErrorReason && SESSION_ERROR_REASONS.has(authErrorReason)) || authErrorStatus === 500
  )

  const isMaintenanceError = authErrorReason === MAINTENANCE_REASON

  return {
    form,
    onSubmit,
    errorMessage,
    isSessionError,
    isMaintenanceError,
    resetAuthenticate,
  }
}
