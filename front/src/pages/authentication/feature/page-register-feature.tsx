import { z } from 'zod'
import PageRegister from '../ui/page-register'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { useNavigate, useParams } from 'react-router'
import { useEffect } from 'react'
import { useRegistrationMutation } from '@/api/auth.api'
import { useAuth } from '@/hooks/use-auth'
import { RouterParams } from '@/routes/router'

const registerSchema = z.object({
  username: z.string().min(1),
  email: z.string().email(),
  password: z.string().min(6, 'Password must be at least 6 characters long'),
  confirmPassword: z.string().min(6, 'Confirm Password must be at least 6 characters long'),
  firstName: z.string().optional(),
  lastName: z.string().optional(),
}).refine((data) => data.password === data.confirmPassword, {
  message: 'Passwords do not match',
})

export type RegisterSchema = z.infer<typeof registerSchema>

export default function PageRegisterFeature() {
  const navigate = useNavigate()
  const { realm_name } = useParams<RouterParams>()
  const { mutate: registration, data } = useRegistrationMutation()
  const { setAuthTokens } = useAuth()

  const backToLogin = () => {
    navigate('../login')
  }

  const form = useForm<RegisterSchema>({
    resolver: zodResolver(registerSchema),
  })

  function onSubmit(data: RegisterSchema) {
    registration({
      body: {
        email: data.email,
        first_name: data.firstName,
        last_name: data.lastName,
        password: data.password,
        username: data.username
      },
      path: {
        realm_name: realm_name ?? 'master'
      }
    })
  }

  useEffect(() => {
    if (data) {
      setAuthTokens(data.access_token, data.refresh_token)
      navigate(`/realms/${realm_name}/overview`, { replace: true })
    }
  }, [data, setAuthTokens, navigate, realm_name])

  return (
    <PageRegister form={form} onSubmit={onSubmit} backToLogin={backToLogin} />
  )
}
