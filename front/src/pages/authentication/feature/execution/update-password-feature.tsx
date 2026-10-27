import { useNavigate, useParams } from 'react-router'
import type { RouterParams } from '@/routes/router.ts'
import UpdatePassword from '@/pages/authentication/ui/execution/update-password.tsx'
import { useUpdatePassword } from '@/api/trident.api'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import type {
  UpdatePasswordSchema} from '../../schemas/update-password.schema'
import {
  buildUpdatePasswordSchema
} from '../../schemas/update-password.schema'
import { Form } from '@/components/ui/form'
import { useMemo } from 'react'
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'
import { usePublicPasswordPolicy } from '@/api/password-policy.api'
import { passwordPolicyRequirements } from '@/lib/password-policy'
import { apiErrorMessage, partitionFieldErrors, validationErrorsFrom } from '@/lib/api-error'
import { AUTH_NAMESPACE } from '../../constants'
import { continueLogin } from '../../utils/continue-login'

const FIELD_BY_API_FIELD: Record<string, keyof UpdatePasswordSchema> = {
  password: 'password',
  value: 'password',
}

export default function UpdatePasswordFeature() {
  const { realm_name } = useParams<RouterParams>()
  const { t } = useTranslation(AUTH_NAMESPACE)
  const { mutate: updatePassword } = useUpdatePassword()
  const navigate = useNavigate()

  const { data: passwordPolicy } = usePublicPasswordPolicy(realm_name)
  const schema = useMemo(() => buildUpdatePasswordSchema(passwordPolicy), [passwordPolicy])
  const requirements = passwordPolicyRequirements(passwordPolicy)

  const form = useForm<UpdatePasswordSchema>({
    resolver: zodResolver(schema),
    defaultValues: {
      password: '',
      confirmPassword: ''
    }
  })

  const handleClick = form.handleSubmit((payload) => {
    updatePassword(
      {
        realm: realm_name ?? 'master',
        data: {
          value: payload.password,
        },
      },
      {
        onSuccess: (response) => continueLogin(response, navigate, realm_name),
        onError: (error) => {
          const fieldErrors = validationErrorsFrom(error)
          const { byField, unattached } = partitionFieldErrors(
            fieldErrors,
            (field) => FIELD_BY_API_FIELD[field]
          )

          for (const [field, message] of byField) {
            form.setError(field, { type: 'server', message })
          }

          if (fieldErrors.length === 0 || unattached.length > 0) {
            toast.error(
              unattached.join(' — ') || apiErrorMessage(error, t('update_password.failed'))
            )
          }
        },
      }
    )
  })

  return (
    <Form {...form}>
      <UpdatePassword handleClick={handleClick} requirements={requirements} />
    </Form>
  )
}
