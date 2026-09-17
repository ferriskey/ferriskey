import { useResetPassword, useVerifyResetToken } from '@/api/password-reset.api'
import { Form } from '@/components/ui/form'
import { useAuth } from '@/hooks/use-auth'
import { zodResolver } from '@hookform/resolvers/zod'
import { useEffect, useMemo, useState } from 'react'
import { useForm } from 'react-hook-form'
import { useLocation, useNavigate, useParams } from 'react-router'
import { buildResetPasswordSchema, type ResetPasswordSchema } from '../schemas/reset-password.schema'
import PageResetPassword from '../ui/page-reset-password'
import { usePublicPasswordPolicy, DEFAULT_PASSWORD_POLICY } from '@/api/password-policy.api'
import { apiErrorMessage, partitionFieldErrors, validationErrorsFrom } from '@/lib/api-error'

type TokenStatus = 'loading' | 'valid' | 'invalid'

const FIELD_BY_API_FIELD: Record<string, keyof ResetPasswordSchema> = {
  password: 'password',
  new_password: 'password',
  value: 'password',
}

export default function PageResetPasswordFeature() {
  const { realm_name } = useParams()
  const navigate = useNavigate()
  const location = useLocation()
  const searchParams = useMemo(() => new URLSearchParams(location.search), [location.search])

  const tokenId = searchParams.get('token_id')
  const token = searchParams.get('token')
  const missingParams = !tokenId || !token

  const [tokenStatus, setTokenStatus] = useState<TokenStatus>(missingParams ? 'invalid' : 'loading')
  const [errorMessage, setErrorMessage] = useState<string | null>(null)
  const { mutate: verifyToken } = useVerifyResetToken()
  const { mutate: resetPassword, isPending } = useResetPassword()
  const { setAuthTokens } = useAuth()

  const { data: policy, isLoading: isPolicyLoading } = usePublicPasswordPolicy(realm_name)
  const resolvedPolicy = policy ?? DEFAULT_PASSWORD_POLICY

  const resetPasswordSchema = useMemo(
    () => buildResetPasswordSchema(resolvedPolicy),
    [resolvedPolicy],
  )

  const form = useForm<ResetPasswordSchema>({
    resolver: zodResolver(resetPasswordSchema),
    defaultValues: { password: '', confirmPassword: '' },
  })

  // Re-validate when policy arrives after the user may have already typed
  useEffect(() => {
    if (!isPolicyLoading) {
      const current = form.getValues('password')
      if (current) {
        void form.trigger('password')
      }
    }
  }, [isPolicyLoading, resolvedPolicy, form])

  useEffect(() => {
    if (missingParams) return

    verifyToken(
      {
        path: { realm_name: realm_name ?? 'master' },
        body: { token_id: tokenId },
      },
      {
        onSuccess: () => setTokenStatus('valid'),
        onError: () => setTokenStatus('invalid'),
      }
    )
  }, []) // eslint-disable-line react-hooks/exhaustive-deps

  function onSubmit(data: ResetPasswordSchema) {
    if (missingParams) return

    setErrorMessage(null)

    resetPassword(
      {
        path: { realm_name: realm_name ?? 'master' },
        body: {
          token_id: tokenId,
          token: token,
          new_password: data.password,
        },
      },
      {
        onSuccess: (data) => {
          setAuthTokens(data.access_token, data.refresh_token, data.id_token ?? null)

          if (data.login_url) {
            window.location.href = data.login_url
            return
          }

          navigate(`/realms/${realm_name}/overview`)
        },
        onError: (error) => {
          const { byField, unattached } = partitionFieldErrors(
            validationErrorsFrom(error),
            (field) => FIELD_BY_API_FIELD[field]
          )

          for (const [field, message] of byField) {
            form.setError(field, { type: 'server', message })
          }

          if (byField.size > 0 && unattached.length === 0) return

          setErrorMessage(
            unattached.join(' — ') || apiErrorMessage(error, 'Could not reset your password.')
          )
        },
      }
    )
  }

  return (
    <Form {...form}>
      <PageResetPassword
        form={form}
        onSubmit={onSubmit}
        isPending={isPending}
        tokenStatus={tokenStatus}
        errorMessage={errorMessage}
        policy={resolvedPolicy}
      />
    </Form>
  )
}
