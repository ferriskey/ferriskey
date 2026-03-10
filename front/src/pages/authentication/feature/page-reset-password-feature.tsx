import { useResetPassword } from '@/api/password-reset.api'
import { Form } from '@/components/ui/form'
import { zodResolver } from '@hookform/resolvers/zod'
import { useMemo, useState } from 'react'
import { useForm } from 'react-hook-form'
import { useLocation, useParams } from 'react-router'
import { resetPasswordSchema, type ResetPasswordSchema } from '../schemas/reset-password.schema'
import PageResetPassword from '../ui/page-reset-password'

export default function PageResetPasswordFeature() {
  const { realm_name } = useParams()
  const location = useLocation()
  const searchParams = useMemo(() => new URLSearchParams(location.search), [location.search])

  const tokenId = searchParams.get('token_id')
  const token = searchParams.get('token')
  const missingParams = !tokenId || !token

  const [success, setSuccess] = useState(false)
  const { mutate: resetPassword, isPending, error } = useResetPassword()

  const form = useForm<ResetPasswordSchema>({
    resolver: zodResolver(resetPasswordSchema),
    defaultValues: { password: '', confirmPassword: '' },
  })

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
      { onSuccess: () => setSuccess(true) }
    )
  }

  return (
    <Form {...form}>
      <PageResetPassword
        form={form}
        onSubmit={onSubmit}
        success={success}
        isPending={isPending}
        missingParams={missingParams}
        errorMessage={error?.message ?? null}
      />
    </Form>
  )
}
