import { GrantType } from '@/api/core.interface'
import { useTokenMutation } from '@/api/auth.api'
import { useAuth } from '@/hooks/use-auth'
import { useEffect, useMemo, useRef, useState } from 'react'
import { useNavigate, useParams } from 'react-router-dom'
import PageCallback from '../ui/page-callback'
import {
  buildLoginErrorRedirect,
  getTokenExchangeErrorMessage,
  validateCallbackParams,
} from './callback-helpers'
import { POST_LOGIN_RETURN_KEY } from './page-device-verify-feature'

// Only allow returning to in-app paths to avoid an open-redirect via the
// sessionStorage channel.
function safePostLoginReturn(realm: string): string | null {
  try {
    const raw = sessionStorage.getItem(POST_LOGIN_RETURN_KEY)
    if (!raw) return null
    sessionStorage.removeItem(POST_LOGIN_RETURN_KEY)
    if (!raw.startsWith(`/realms/${realm}/`)) return null
    return raw
  } catch {
    return null
  }
}

export default function PageCallbackFeature() {
  const navigate = useNavigate()

  const urlParams = useMemo(() => new URLSearchParams(window.location.search), [])
  const code = useMemo(() => {
    return urlParams.get('code')
  }, [urlParams])
  const state = useMemo(() => urlParams.get('state'), [urlParams])
  const setup = true

  const { realm_name } = useParams()
  const { setAuthTokens } = useAuth()

  const { mutateAsync: exchangeToken } = useTokenMutation()
  const hasStartedExchange = useRef(false)

  // Read storage once at mount to avoid a React StrictMode double-render
  // causing the value to be missing on the second render after the first effect
  // removes it. Keyed by the returned state so concurrent flows don't collide.
  const [expectedState] = useState(() =>
    state ? localStorage.getItem(`oauth_state:${state}`) : null
  )

  const callbackValidationError = useMemo(() => {
    return validateCallbackParams({
      code,
      returnedState: state,
      expectedState,
    })
  }, [code, state, expectedState])

  useEffect(() => {
    if (callbackValidationError) {
      if (state) localStorage.removeItem(`oauth_state:${state}`)
      document.cookie = 'FERRISKEY_SESSION=; expires=Thu, 01 Jan 1970 00:00:00 GMT; path=/;'
      navigate(buildLoginErrorRedirect(realm_name, callbackValidationError), { replace: true })
      return
    }

    if (!code || hasStartedExchange.current) {
      return
    }

    hasStartedExchange.current = true
    if (state) localStorage.removeItem(`oauth_state:${state}`)

    void exchangeToken({
      realm: realm_name ?? 'master',
      data: {
        client_id: 'security-admin-console',
        code,
        grant_type: GrantType.Code,
      },
    })
      .then((data) => {
        setAuthTokens(data.access_token, data.refresh_token, data.id_token ?? null)
        const realm = realm_name ?? 'master'
        const returnTo = safePostLoginReturn(realm)
        navigate(returnTo ?? `/realms/${realm}/overview`, { replace: true })
      })
      .catch((error: unknown) => {
        const message = getTokenExchangeErrorMessage(error)
        document.cookie = 'FERRISKEY_SESSION=; expires=Thu, 01 Jan 1970 00:00:00 GMT; path=/;'
        navigate(buildLoginErrorRedirect(realm_name, message), { replace: true })
      })
  }, [callbackValidationError, code, exchangeToken, navigate, realm_name, setAuthTokens, state])

  return <PageCallback code={code} setup={setup} />
}
