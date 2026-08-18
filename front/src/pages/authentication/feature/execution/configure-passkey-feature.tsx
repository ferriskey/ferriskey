import { useAuthenticateMutation } from '@/api/auth.api'
import { useCallback, useEffect, useState } from 'react'
import { useNavigate, useParams } from 'react-router'
import { RouterParams } from '@/routes/router'
import { toast } from 'sonner'
import { AuthenticationStatus } from '@/api/api.interface'
import ConfigurePasskey from '../../ui/execution/configure-passkey'
import { isWebAuthnAvailable, startRegistration } from '@/lib/webauthn'

export default function ConfigurePasskeyFeature() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const {
    mutate: authenticate,
    data: authenticateData,
  } = useAuthenticateMutation()

  const [isLoading, setIsLoading] = useState(false)
  const [isSuccess, setIsSuccess] = useState(false)

  const completeAuth = useCallback(() => {
    authenticate({
      clientId: 'security-admin-console',
      realm: realm_name ?? 'master',
      data: {},
    })
  }, [authenticate, realm_name])

  const onRegister = useCallback(async () => {
    if (!isWebAuthnAvailable()) {
      toast.error('WebAuthn is not supported in this browser')
      return
    }

    setIsLoading(true)
    try {
      // Step 1: Get creation options
      const optionsRes = await window.tanstackApi.client.post(
        '/realms/{realm_name}/login-actions/webauthn-public-key-create-options',
        {
          path: { realm_name: realm_name ?? 'master' },
        } as never,
      )

      // Step 2: Create credential with browser
      const credential = await startRegistration((optionsRes as { publicKey: Record<string, unknown> }).publicKey)

      // Step 3: Send credential to server
      await window.tanstackApi.client.post(
        '/realms/{realm_name}/login-actions/webauthn-public-key-create',
        {
          path: { realm_name: realm_name ?? 'master' },
          body: credential,
        } as never,
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
  }, [realm_name, completeAuth])

  useEffect(() => {
    if (!authenticateData) return
    if (authenticateData.url) {
      window.location.href = authenticateData.url
    }

    if (
      authenticateData.status === AuthenticationStatus.RequiresActions &&
      authenticateData.required_actions &&
      authenticateData.required_actions.length > 0
    ) {
      const firstRequiredAction = authenticateData.required_actions[0]

      navigate(
        `/realms/${realm_name}/authentication/required-action?execution=${firstRequiredAction.toUpperCase()}`
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
