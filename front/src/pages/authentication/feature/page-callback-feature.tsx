import { GrantType } from '@/api/api.interface'
import { useTokenMutation } from '@/api/auth.api'
import { useAuth } from '@/hooks/use-auth'
import { useEffect, useState } from 'react'
import { useNavigate, useParams } from 'react-router-dom'
import PageCallback from '../ui/page-callback'

export default function PageCallbackFeature() {
  const [code, setCode] = useState<string | null>(null)
  const [setup, setSetup] = useState<boolean>(false)
  const { realm_name } = useParams()
  const { setAuthToken } = useAuth()
  const navigate = useNavigate()

  const { mutate: exchangeToken, data, status } = useTokenMutation()

  useEffect(() => {
    const urlParams = new URLSearchParams(window.location.search)
    console.log(urlParams, realm_name)
    const code = urlParams.get('code')
    setCode(code)

    setSetup(true)
  }, [])

  useEffect(() => {
    if (code && setup) {
      exchangeToken({
        data: {
          client_id: 'security-admin-console',
          code: code,
          grant_type: GrantType.Code,
        },
        realm: realm_name ?? 'master',
      })
    }
  }, [code, setup])

  useEffect(() => {
    if (data) {
      setAuthToken(data.access_token)

      navigate(`/realms/${realm_name}/overview`)
    }
  }, [data])

  return <PageCallback code={code} setup={setup} />
}
