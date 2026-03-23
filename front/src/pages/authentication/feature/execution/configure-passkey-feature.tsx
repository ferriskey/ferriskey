import { useAuthenticateMutation } from '@/api/auth.api'
import { useCallback, useEffect, useState } from 'react'
import { useNavigate, useParams, useSearchParams } from 'react-router'
import { RouterParams } from '@/routes/router'
import { toast } from 'sonner'
import { AuthenticationStatus } from '@/api/api.interface'
import ConfigurePasskey from '../../ui/execution/configure-passkey'
import { isWebAuthnAvailable, startRegistration } from '@/lib/webauthn'

export default function ConfigurePasskeyFeature() {
  const { realm_name } = useParams<RouterParams>()
  const [searchParams] = useSearchParams()
  const navigate = useNavigate()
  const {
    mutate: authenticate,
    data: authenticateData,
  } = useAuthenticateMutation()

  const token = searchParams.get('client_data')
  const [isLoading, setIsLoading] = useState(false)
  const [isSuccess, setIsSuccess] = useState(false)

  useEffect(() => {
    if (!token) {
      toast.error('Token is missing')
      navigate(`/realms/${realm_name}/authentication/login`)
    }
  }, [token, navigate, realm_name])

  const completeAuth = useCallback(() => {
    authenticate({
      clientId: 'security-admin-console',
      realm: realm_name ?? 'master',
      data: {},
      useToken: true,
      token: token ?? undefined,
    })
  }, [authenticate, realm_name, token])

  const onRegister = useCallback(async () => {
    if (!isWebAuthnAvailable()) {
      toast.error('WebAuthn is not supported in this browser')
      return
    }

    if (!token) return

    setIsLoading(true)
    try {
      // Step 1: Get creation options
      const optionsRes = await window.axios.post(
        `/realms/${realm_name}/login-actions/webauthn-public-key-create-options`,
        {},
        { headers: { Authorization: `Bearer ${token}` } }
      )

      // Step 2: Create credential with browser
      const credential = await startRegistration(optionsRes.data.publicKey)

      // Step 3: Send credential to server
      await window.axios.post(
        `/realms/${realm_name}/login-actions/webauthn-public-key-create`,
        credential,
        { headers: { Authorization: `Bearer ${token}` } }
      )

      setIsSuccess(true)
      toast.success('Passkey registered successfully')

      // Step 4: Complete authentication
      setTimeout(() => completeAuth(), 1000)
    } catch (err) {
      console.error('Passkey registration failed:', err)
      if (err instanceof DOMException && (err.name === 'NotAllowedError' || err.name === 'InvalidStateError')) {
        toast.info('A passkey already exists for this account. Redirecting...')
        setTimeout(() => completeAuth(), 1000)
      } else {
        toast.error('Passkey registration failed')
      }
    } finally {
      setIsLoading(false)
    }
  }, [realm_name, token, completeAuth])

  useEffect(() => {
    if (!authenticateData) return
    if (authenticateData.url) {
      window.location.href = authenticateData.url
    }

    if (
      authenticateData.status === AuthenticationStatus.RequiresActions &&
      authenticateData.required_actions &&
      authenticateData.required_actions.length > 0 &&
      authenticateData.token
    ) {
      const firstRequiredAction = authenticateData.required_actions[0]

      navigate(
        `/realms/${realm_name}/authentication/required-action?execution=${firstRequiredAction.toUpperCase()}&client_data=${authenticateData.token}`
      )
    }
  }, [authenticateData, navigate, realm_name])

  return (
    <ConfigurePasskey
      onRegister={onRegister}
      isLoading={isLoading}
      isSuccess={isSuccess}
    />
  )
}
