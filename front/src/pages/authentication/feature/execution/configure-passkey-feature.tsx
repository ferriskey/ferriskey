import { useAuthenticateMutation } from '@/api/auth.api'
import { useCallback, useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useNavigate, useParams } from 'react-router'
import { RouterParams } from '@/routes/router'
import { toast } from 'sonner'
import { AuthenticationStatus } from '@/api/api.interface'
import ConfigurePasskey from '../../ui/execution/configure-passkey'
import { isWebAuthnAvailable, startRegistration } from '@/lib/webauthn'
import { AUTH_NAMESPACE } from '../../constants'

export default function ConfigurePasskeyFeature() {
  const { realm_name } = useParams<RouterParams>()
  const { t } = useTranslation(AUTH_NAMESPACE)
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
      toast.error(t('passkey.unsupported'))
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
      toast.success(t('configure_passkey.success'))

      // Step 4: Complete authentication
      setTimeout(() => completeAuth(), 1000)
    } catch (err) {
      console.error('Passkey registration failed:', err)
      if (err instanceof DOMException && (err.name === 'NotAllowedError' || err.name === 'InvalidStateError')) {
        toast.info(t('configure_passkey.already_exists'))
        setTimeout(() => completeAuth(), 1000)
      } else {
        toast.error(t('configure_passkey.failed'))
      }
    } finally {
      setIsLoading(false)
    }
  }, [realm_name, completeAuth, t])

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
