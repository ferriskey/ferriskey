import { useEffect, useMemo, useState } from 'react'
import { useLocation, useParams } from 'react-router'
import PageMagicLinkVerify from '../ui/page-magic-link-verify'

type VerifyStatus = 'loading' | 'error'

export default function PageMagicLinkVerifyFeature() {
  const { realm_name } = useParams()
  const location = useLocation()
  const searchParams = useMemo(() => new URLSearchParams(location.search), [location.search])

  const tokenId = searchParams.get('token_id')
  const magicToken = searchParams.get('magic_token')
  const missingParams = !tokenId || !magicToken

  const [status, setStatus] = useState<VerifyStatus>(missingParams ? 'error' : 'loading')
  const [errorMessage, setErrorMessage] = useState<string | null>(
    missingParams ? 'Missing magic link parameters.' : null
  )

  useEffect(() => {
    if (missingParams) return

    const realm = realm_name ?? 'master'

    const ensureSession = async () => {
      const state = crypto.randomUUID()
      sessionStorage.setItem('oauth_state', state)
      const callbackUrl = `${window.location.origin}/realms/${realm}/authentication/callback`

      const query = new URLSearchParams({
        response_type: 'code',
        client_id: 'security-admin-console',
        redirect_uri: callbackUrl,
        scope: 'openid profile email',
        state,
      }).toString()

      await fetch(`${window.apiUrl}/realms/${realm}/protocol/openid-connect/auth?${query}`, {
        credentials: 'include',
        redirect: 'manual',
      })
    }

    const verify = async () => {
      const url = `${window.apiUrl}/realms/${realm}/login-actions/verify-magic-link?token_id=${tokenId}&magic_token=${encodeURIComponent(magicToken)}`
      const res = await fetch(url, { credentials: 'include' })

      if (!res.ok) {
        const body = await res.json().catch(() => ({}))
        throw new Error((body as { message?: string }).message ?? 'Verification failed')
      }

      const data: { url?: string } = await res.json()
      if (data.url) {
        window.location.href = data.url
      } else {
        throw new Error('Verification succeeded but no redirect URL was provided.')
      }
    }

    ensureSession()
      .then(() => verify())
      .catch((err: Error) => {
        setErrorMessage(err.message)
        setStatus('error')
      })
  }, []) // eslint-disable-line react-hooks/exhaustive-deps

  return <PageMagicLinkVerify status={status} errorMessage={errorMessage} />
}
