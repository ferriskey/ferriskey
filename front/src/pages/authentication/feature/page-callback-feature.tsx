import { GrantType } from '@/api/core.interface'
import { useTokenMutation } from '@/api/auth.api'
import { useAuth } from '@/hooks/use-auth'
import { useEffect, useMemo, useRef } from 'react'
import { useNavigate, useParams } from 'react-router-dom'
import PageCallback from '../ui/page-callback'

export default function PageCallbackFeature() {
  const navigate = useNavigate()

  const code = useMemo(() => {
    const urlParams = new URLSearchParams(window.location.search)
    return urlParams.get('code')
  }, [])
  const setup = true

  const { realm_name } = useParams()
  const { setAuthTokens } = useAuth()

  const { mutate: exchangeToken, data, error } = useTokenMutation()
  const hasProcessedToken = useRef(false)

  useEffect(() => {
    if (code && !hasProcessedToken.current) {
      exchangeToken({
        realm: realm_name ?? 'master',
        data: {
          client_id: 'security-admin-console',
          code,
          grant_type: GrantType.Code,
        },
      })
    }
  }, [code, exchangeToken, realm_name])

  useEffect(() => {
    if (data && !hasProcessedToken.current) {
      hasProcessedToken.current = true

      setAuthTokens(data.access_token, data.refresh_token, data.id_token ?? null)

      navigate(`/realms/${realm_name ?? 'master'}/overview`, { replace: true })
    }
  }, [data, realm_name, navigate, setAuthTokens])

  useEffect(() => {
    if (error && !hasProcessedToken.current) {
      document.cookie = 'FERRISKEY_SESSION=; expires=Thu, 01 Jan 1970 00:00:00 GMT; path=/;'
      hasProcessedToken.current = true
    }
  }, [error, hasProcessedToken])

  return <PageCallback code={code} setup={setup} />
}
