import { useNavigate, useParams, useSearchParams } from 'react-router'
import { RouterParams } from '@/routes/router.ts'
import UpdatePassword from '@/pages/authentication/ui/execution/update-password.tsx'
import { useUpdatePassword } from '@/api/trident.api'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { updatePasswordSchema, UpdatePasswordSchema } from '../../schemas/update-password.schema'
import { Form } from '@/components/ui/form'
import { useEffect } from 'react'
import { useAuthenticateMutation } from '@/api/auth.api'
import { AuthenticationStatus } from '@/api/api.interface'


export default function UpdatePasswordFeature() {
  const { realm_name } = useParams<RouterParams>()
  const [searchParams] = useSearchParams()
  const { mutate: updatePassword, data: responseUpdatePassword } = useUpdatePassword()
  const { mutate: authenticate, data: authenticateResponse } = useAuthenticateMutation()
  const navigate = useNavigate()
  const token = searchParams.get('client_data')

  const form = useForm<UpdatePasswordSchema>({
    resolver: zodResolver(updatePasswordSchema),
    defaultValues: {
      password: '',
      confirmPassword: ''
    }
  })

  const handleClick = form.handleSubmit((payload) => {
    updatePassword({
      body: {
        value: payload.password,
      }
    })
  })

  useEffect(() => {
    if (responseUpdatePassword) {
      const cookies = document.cookie.split(';').reduce(
        (acc, cookie) => {
          const [key, value] = cookie.trim().split('=')
          acc[key] = value
          return acc
        },
        {} as Record<string, string>
      )

      const sessionCode = cookies['FERRISKEY_SESSION'] || ''
      authenticate({
        clientId: 'security-admin-console',
        realm: realm_name ?? 'master',
        data: {},
        sessionCode,
        token: token ?? undefined
      })
    }
  }, [responseUpdatePassword, authenticate, realm_name, token])

  useEffect(() => {
    if (!authenticateResponse) return
    if (authenticateResponse.url) {
      window.location.href = authenticateResponse.url
    }

    if (
      authenticateResponse.status === AuthenticationStatus.RequiresActions &&
      authenticateResponse.required_actions &&
      authenticateResponse.required_actions.length > 0 &&
      authenticateResponse.token
    ) {
      const firstRequiredAction = authenticateResponse.required_actions[0]

      navigate(
        `/realms/${realm_name}/authentication/required-action?execution=${firstRequiredAction.toUpperCase()}&client_data=${authenticateResponse.token}`
      )
    }
  }, [authenticateResponse, navigate, realm_name])


  return (
    <Form {...form}>
      <UpdatePassword handleClick={handleClick} />
    </Form>
  )
}
