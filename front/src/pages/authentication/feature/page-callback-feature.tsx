import { GrantType } from '@/api/api.interface'
import { useTokenMutation } from '@/api/auth.api'
import { useAuth } from '@/hooks/use-auth'
import { useEffect, useRef, useState } from 'react'
import { useNavigate, useParams } from 'react-router-dom'
import PageCallback from '../ui/page-callback'

export default function PageCallbackFeature() {
  const navigate = useNavigate()

  const [code, setCode] = useState<string | null>(null)
  const [setup, setSetup] = useState<boolean>(false)

  const { realm_name } = useParams()
  const { setAuthTokens } = useAuth()

  const { mutate: exchangeToken, data } = useTokenMutation()
  const hasProcessedToken = useRef(false)

  useEffect(() => {
    const urlParams = new URLSearchParams(window.location.search)
    const codeParam = urlParams.get('code')

    if (!setup) {
      setCode(codeParam)
      setSetup(true)
    }
  }, [])

  useEffect(() => {
    if (code && setup && !hasProcessedToken.current) {
      exchangeToken({
        realm: realm_name ?? 'master',
        data: {
          client_id: 'security-admin-console',
          code: code,
          grant_type: GrantType.Code,
        },
      })
    }
  }, [code, setup, exchangeToken, realm_name])

  useEffect(() => {
    if (data && !hasProcessedToken.current) {
      hasProcessedToken.current = true

      setAuthTokens(data.access_token, data.refresh_token)

      navigate(`/realms/${realm_name ?? 'master'}/overview`, { replace: true })
    }
  }, [data, realm_name, navigate, setAuthTokens])

  return <PageCallback code={code} setup={setup} />
}