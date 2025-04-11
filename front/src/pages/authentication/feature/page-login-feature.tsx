import { useAuthenticateMutation, useAuthQuery } from '@/api/auth.api'
import { zodResolver } from '@hookform/resolvers/zod'
import { useEffect, useState } from 'react'
import { useForm } from 'react-hook-form'
import { useNavigate } from 'react-router'
import { z } from 'zod'
import PageLogin from '../ui/page-login'

export const initiateOAuthLogin = async (realmName: string) => {
  const params = new URLSearchParams({
    response_type: 'code',
    client_id: 'security-admin-console',
    redirect_uri: 'http://localhost:5173/realms/master/authentication/callback', // URL de callback de votre app
    scope: 'openid profile email',
    state: crypto.randomUUID(), // Générer un état unique pour la sécurité
  })

  const authUrl = `http://localhost:3333/realms/${realmName}/protocol/openid-connect/auth?${params.toString()}`
  window.location.href = authUrl
}

export const initiateOAuthLoginQuery = (realmName: string) => {
  const params = new URLSearchParams({
    response_type: 'code',
    client_id: 'security-admin-console',
    redirect_uri: 'http://localhost:5173/realms/master/authentication/callback', // URL de callback de votre app
    scope: 'openid profile email',
    state: crypto.randomUUID(), // Générer un état unique pour la sécurité
  })

  return {
    query: params.toString(),
    realm: realmName,
  }
}

// J'aimerais que si j'ai une erreur de connexion, je puisse afficher un message d'erreur proprement

const authenticateSchema = z.object({
  username: z.string().min(1),
  password: z.string().min(1),
})

export type AuthenticateSchema = z.infer<typeof authenticateSchema>

export default function PageLoginFeature() {
  const navigate = useNavigate()
  const [isAuthInitiated, setIsAuthInitiated] = useState<boolean>(false)
  const [isSetup, setIsSetup] = useState(false)

  const getOAuthParams = () => {
    const state = crypto.randomUUID()

    sessionStorage.setItem('oauth_state', state)

    return {
      query: new URLSearchParams({
        response_type: 'code',
        client_id: 'security-admin-console',
        redirect_uri: 'http://localhost:5173/realms/master/authentication/callback', // URL de callback de votre app
        scope: 'openid profile email',
        state,
      }).toString(),
      realm: 'master',
    }
  }

  //const { data, isSuccess, isError, error } = useAuthQuery(getOAuthParams())c
  const { data, isSuccess, isError, error } = useAuthQuery(getOAuthParams())
  const { mutate: authenticate } = useAuthenticateMutation()

  const form = useForm<AuthenticateSchema>({
    resolver: zodResolver(authenticateSchema),
    defaultValues: {
      username: '',
      password: '',
    },
  })

  const onSubmit = (data: AuthenticateSchema) => {
    // Get session_code from cookies
    console.log('cookies', document.cookie)
    const cookies = document.cookie.split(';').reduce(
      (acc, cookie) => {
        const [key, value] = cookie.trim().split('=')
        acc[key] = value
        return acc
      },
      {} as Record<string, string>
    )

    const sessionCode = cookies['session_code'] || '123456' // Fallback to default if not found
    authenticate({
      data,
      realm: 'master',
      clientId: 'security-admin-console',
      sessionCode,
    })
  }

  useEffect(() => {
    const urlParams = new URLSearchParams(window.location.search)
    const clientId = urlParams.get('client_id')
    const redirectUri = urlParams.get('redirect_uri')

    if (clientId && redirectUri) {
      setIsAuthInitiated(true)
    }

    setIsSetup(true)
  }, [])

  useEffect(() => {
    if (data && isSetup && !isAuthInitiated) {
      navigate(data.url)
    }
  }, [data, isSetup])

  return <PageLogin form={form} onSubmit={onSubmit} isError={isError} />
}
