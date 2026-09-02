import { useNavigate, useParams } from 'react-router'
import { RouterParams } from '@/routes/router.ts'
import UpdatePassword from '@/pages/authentication/ui/execution/update-password.tsx'
import { useUpdatePassword } from '@/api/trident.api'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import {
  buildUpdatePasswordSchema,
  UpdatePasswordSchema,
} from '../../schemas/update-password.schema'
import { Form } from '@/components/ui/form'
import { useEffect, useMemo } from 'react'
import { toast } from 'sonner'
import { useAuthenticateMutation } from '@/api/auth.api'
import { AuthenticationStatus } from '@/api/api.interface'
import { usePublicPasswordPolicy } from '@/api/password-policy.api'
import { passwordPolicyRequirements } from '@/lib/password-policy'
import { validationErrorsFrom } from '@/lib/api-error'

const FIELD_BY_API_FIELD: Record<string, keyof UpdatePasswordSchema> = {
  password: 'password',
  value: 'password',
}

export default function UpdatePasswordFeature() {
  const { realm_name } = useParams<RouterParams>()
  const { mutate: updatePassword, data: responseUpdatePassword } = useUpdatePassword()
  const { mutate: authenticate, data: authenticateResponse } = useAuthenticateMutation()
  const navigate = useNavigate()

  const { data: passwordPolicy } = usePublicPasswordPolicy(realm_name)
  const schema = useMemo(() => buildUpdatePasswordSchema(passwordPolicy), [passwordPolicy])
  const requirements = useMemo(
    () => passwordPolicyRequirements(passwordPolicy),
    [passwordPolicy]
  )

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
        onError: (error) => {
          const fieldErrors = validationErrorsFrom(error)
          const unattached: string[] = []

          for (const fieldError of fieldErrors) {
            const field = FIELD_BY_API_FIELD[fieldError.field]
            if (field) {
              form.setError(field, { type: 'server', message: fieldError.message })
            } else {
              unattached.push(fieldError.message)
            }
          }

          if (fieldErrors.length === 0 || unattached.length > 0) {
            toast.error(
              unattached.join(' — ') || error.message || 'Failed to update your password'
            )
          }
        },
      }
    )
  })

  useEffect(() => {
    if (responseUpdatePassword) {
      authenticate({
        clientId: 'security-admin-console',
        realm: realm_name ?? 'master',
        data: {},
      })
    }
  }, [responseUpdatePassword, authenticate, realm_name])

  useEffect(() => {
    if (!authenticateResponse) return
    if (authenticateResponse.url) {
      window.location.href = authenticateResponse.url
    }

    if (
      authenticateResponse.status === AuthenticationStatus.RequiresActions &&
      authenticateResponse.required_actions &&
      authenticateResponse.required_actions.length > 0
    ) {
      const firstRequiredAction = authenticateResponse.required_actions[0]

      navigate(
        `/realms/${realm_name}/authentication/required-action?execution=${firstRequiredAction.toUpperCase()}`
      )
    }
  }, [authenticateResponse, navigate, realm_name])


  return (
    <Form {...form}>
      <UpdatePassword handleClick={handleClick} requirements={requirements} />
    </Form>
  )
}
