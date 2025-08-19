import { useParams, useSearchParams } from 'react-router'
import { RouterParams } from '@/routes/router.ts'
import UpdatePassword from '@/pages/authentication/ui/execution/update-password.tsx'
import { useUpdatePassword } from '@/api/trident.api'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { updatePasswordSchema, UpdatePasswordSchema } from '../../schemas/update-password.schema'
import { Form } from '@/components/ui/form'
import { useEffect } from 'react'
import { useAuthenticateMutation } from '@/api/auth.api'


export default function UpdatePasswordFeature() {
  const { realm_name } = useParams<RouterParams>()
  const [searchParams] = useSearchParams()
  const { mutate: updatePassword, data: responseUpdatePassword } = useUpdatePassword()
  const { mutate: authenticate, data: authenticateResponse } = useAuthenticateMutation()
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
        value: payload.password
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
    if (authenticateResponse && authenticateResponse.url) {
      window.location.href = authenticateResponse.url
    }
  }, [authenticateResponse])


  return (
    <Form {...form}>
      <UpdatePassword handleClick={handleClick} />
    </Form>
  )
}
