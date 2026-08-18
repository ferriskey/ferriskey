import { useNavigate, useParams } from 'react-router'
import { RouterParams } from '@/routes/router.ts'
import UpdatePassword from '@/pages/authentication/ui/execution/update-password.tsx'
import { useUpdatePassword } from '@/api/trident.api'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { updatePasswordSchema, UpdatePasswordSchema } from '../../schemas/update-password.schema'
import { Form } from '@/components/ui/form'
import { useEffect } from 'react'
import { toast } from 'sonner'
import { useAuthenticateMutation } from '@/api/auth.api'
import { AuthenticationStatus } from '@/api/api.interface'


export default function UpdatePasswordFeature() {
  const { realm_name } = useParams<RouterParams>()
  const { mutate: updatePassword, data: responseUpdatePassword } = useUpdatePassword()
  const { mutate: authenticate, data: authenticateResponse } = useAuthenticateMutation()
  const navigate = useNavigate()

  const form = useForm<UpdatePasswordSchema>({
    resolver: zodResolver(updatePasswordSchema),
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
          toast.error(error.message || 'Failed to update your password')
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
      <UpdatePassword handleClick={handleClick} />
    </Form>
  )
}
