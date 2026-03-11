import { useResetPassword, useVerifyResetToken } from '@/api/password-reset.api'
import { Form } from '@/components/ui/form'
import { useAuth } from '@/hooks/use-auth'
import { zodResolver } from '@hookform/resolvers/zod'
import { useEffect, useMemo, useState } from 'react'
import { useForm } from 'react-hook-form'
import { useLocation, useNavigate, useParams } from 'react-router'
import { resetPasswordSchema, type ResetPasswordSchema } from '../schemas/reset-password.schema'
import PageResetPassword from '../ui/page-reset-password'

type TokenStatus = 'loading' | 'valid' | 'invalid'

export default function PageResetPasswordFeature() {
  const { realm_name } = useParams()
  const navigate = useNavigate()
  const location = useLocation()
  const searchParams = useMemo(() => new URLSearchParams(location.search), [location.search])

  const tokenId = searchParams.get('token_id')
  const token = searchParams.get('token')
  const missingParams = !tokenId || !token

  const [tokenStatus, setTokenStatus] = useState<TokenStatus>(missingParams ? 'invalid' : 'loading')
  const { mutate: verifyToken } = useVerifyResetToken()
  const { mutate: resetPassword, isPending, error } = useResetPassword()
  const { setAuthTokens } = useAuth()

  const form = useForm<ResetPasswordSchema>({
    resolver: zodResolver(resetPasswordSchema),
    defaultValues: { password: '', confirmPassword: '' },
  })

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
          navigate(`/realms/${realm_name}/overview`)
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
        errorMessage={error?.message ?? null}
      />
    </Form>
  )
}
