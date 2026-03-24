import { useEffect, useMemo, useRef, useState } from 'react'
import { useLocation, useNavigate, useParams } from 'react-router'
import { useVerifyMagicLink } from '@/api/trident.api'
import PageMagicLinkVerify from '../ui/page-magic-link-verify'

type VerifyStatus = 'loading' | 'error'

export default function PageMagicLinkVerifyFeature() {
  const { realm_name } = useParams()
  const navigate = useNavigate()
  const location = useLocation()
  const searchParams = useMemo(() => new URLSearchParams(location.search), [location.search])

  const tokenId = searchParams.get('token_id')
  const magicToken = searchParams.get('magic_token')
  const missingParams = !tokenId || !magicToken

  // client_id is present after the OAuth redirect loop sets the session cookie
  const isSessionReady = searchParams.has('client_id')

  const [status, setStatus] = useState<VerifyStatus>(missingParams ? 'error' : 'loading')
  const [errorMessage, setErrorMessage] = useState<string | null>(
    missingParams ? 'Missing magic link parameters.' : null
  )

  const { mutateAsync: verifyMagicLink } = useVerifyMagicLink()
  const hasStartedVerification = useRef(false)

  useEffect(() => {
    if (missingParams) return

    const realm = realm_name ?? 'master'

    if (!isSessionReady) {
      sessionStorage.setItem(
        'magic_link_pending',
        JSON.stringify({ token_id: tokenId, magic_token: magicToken })
      )
      navigate(`/realms/${realm}/authentication/login`, { replace: true })
      return
    }

    if (hasStartedVerification.current) return
    hasStartedVerification.current = true

    void verifyMagicLink({
      path: { realm_name: realm },
      query: { token_id: tokenId, magic_token: magicToken },
    })
      .then((data) => {
        if (data.url) {
          window.location.href = data.url
        } else {
          setErrorMessage('Verification succeeded but no redirect URL was provided.')
          setStatus('error')
        }
      })
      .catch((err: Error) => {
        setErrorMessage(err.message)
        setStatus('error')
      })
  }, []) // eslint-disable-line react-hooks/exhaustive-deps

  return <PageMagicLinkVerify status={status} errorMessage={errorMessage} />
}
