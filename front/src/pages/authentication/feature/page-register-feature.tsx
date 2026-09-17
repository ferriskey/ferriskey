import { z } from 'zod'
import PageRegister from '../ui/page-register'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { useNavigate, useParams } from 'react-router'
import { useEffect, useMemo } from 'react'
import { useRegistrationMutation } from '@/api/auth.api'
import { useAuth } from '@/hooks/use-auth'
import { RouterParams } from '@/routes/router'
import { usePublicPasswordPolicy, DEFAULT_PASSWORD_POLICY } from '@/api/password-policy.api'
import { evaluatePassword } from '../utils/password-policy'
import { toast } from 'sonner'
import {
  apiErrorMessage,
  apiErrorReason,
  partitionFieldErrors,
  validationErrorsFrom,
} from '@/lib/api-error'

function buildRegisterSchema(policy: typeof DEFAULT_PASSWORD_POLICY) {
  return z
    .object({
      username: z.string().min(1, 'Username is required'),
      email: z.string().email('Invalid email address'),
      password: z
        .string()
        .min(1, 'Password is required')
        .superRefine((value, ctx) => {
          const result = evaluatePassword(value, policy)
          if (!result.valid) {
            ctx.addIssue({
              code: 'custom',
              message: result.unmetMessages.join(', '),
            })
          }
        }),
      confirmPassword: z.string().min(1, 'Please confirm your password'),
      firstName: z.string().optional(),
      lastName: z.string().optional(),
    })
    .refine((data) => data.password === data.confirmPassword, {
      path: ['confirmPassword'],
      message: 'Passwords do not match',
    })
}

export type RegisterSchema = z.infer<ReturnType<typeof buildRegisterSchema>>

const FIELD_BY_API_FIELD: Record<string, keyof RegisterSchema> = {
  email: 'email',
  username: 'username',
  password: 'password',
  value: 'password',
}

const FIELD_BY_REASON: Record<string, keyof RegisterSchema> = {
  email_already_exists: 'email',
  username_already_exists: 'username',
}

export default function PageRegisterFeature() {
  const navigate = useNavigate()
  const { realm_name } = useParams<RouterParams>()
  const { mutate: registration, data } = useRegistrationMutation()
  const { setAuthTokens } = useAuth()

  const { data: policy, isLoading: isPolicyLoading } = usePublicPasswordPolicy(realm_name)
  const resolvedPolicy = policy ?? DEFAULT_PASSWORD_POLICY

  const registerSchema = useMemo(() => buildRegisterSchema(resolvedPolicy), [resolvedPolicy])

  const backToLogin = () => {
    navigate('../login')
  }

  const form = useForm<RegisterSchema>({
    resolver: zodResolver(registerSchema),
    mode: 'onChange',
    defaultValues: {
      username: '',
      email: '',
      password: '',
      confirmPassword: '',
      firstName: '',
      lastName: '',
    },
  })

  // Re-validate password when policy finishes loading so the checklist is
  // accurate even if the user typed before the policy arrived.
  useEffect(() => {
    if (!isPolicyLoading) {
      const current = form.getValues('password')
      if (current) {
        void form.trigger('password')
      }
    }
  }, [isPolicyLoading, resolvedPolicy, form])

  function onSubmit(data: RegisterSchema) {
    registration(
      {
        body: {
          email: data.email,
          first_name: data.firstName,
          last_name: data.lastName,
          password: data.password,
          username: data.username,
        },
        path: {
          realm_name: realm_name ?? 'master',
        },
      },
      {
        onError: (error) => {
          const { byField, unattached } = partitionFieldErrors(
            validationErrorsFrom(error),
            (field) => FIELD_BY_API_FIELD[field]
          )

          for (const [field, message] of byField) {
            form.setError(field, { type: 'server', message })
          }

          if (byField.size > 0 && unattached.length === 0) return

          const message =
            unattached.join(' — ') ||
            apiErrorMessage(error, 'We could not create your account. Please try again.')
          const field = FIELD_BY_REASON[apiErrorReason(error) ?? '']

          if (field) {
            form.setError(field, { type: 'server', message })
            return
          }

          toast.error(message)
        },
      }
    )
  }

  useEffect(() => {
    if (!data) return

    if (data.status === 'redirect') {
      window.location.href = data.data.url
      return
    }

    if (data.status === 'authenticated') {
      setAuthTokens(
        data.data.access_token,
        data.data.refresh_token,
        data.data.id_token ?? null
      )
      navigate(`/realms/${realm_name}/overview`, { replace: true })
    } else {
      navigate(`/realms/${realm_name}/authentication/check-your-email`, {
        replace: true,
        state: { email: form.getValues('email') },
      })
    }
  }, [data, setAuthTokens, navigate, realm_name, form])

  return (
    <PageRegister
      form={form}
      onSubmit={onSubmit}
      backToLogin={backToLogin}
      policy={resolvedPolicy}
    />
  )
}
