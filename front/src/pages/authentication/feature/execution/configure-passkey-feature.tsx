import { useCallback, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useNavigate, useParams } from 'react-router'
import type { RouterParams } from '@/routes/router'
import { toast } from 'sonner'
import ConfigurePasskey from '../../ui/execution/configure-passkey'
import { isWebAuthnAvailable, startRegistration } from '@/lib/webauthn'
import { AUTH_NAMESPACE } from '../../constants'
import type { Schemas } from '@/api/api.client'
import { continueLogin } from '../../utils/continue-login'

export default function ConfigurePasskeyFeature() {
  const { realm_name } = useParams<RouterParams>()
  const { t } = useTranslation(AUTH_NAMESPACE)
  const navigate = useNavigate()

  const [isLoading, setIsLoading] = useState(false)
  const [isSuccess, setIsSuccess] = useState(false)

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
      const created = (await window.tanstackApi.client.post(
        '/realms/{realm_name}/login-actions/webauthn-public-key-create',
        {
          path: { realm_name: realm_name ?? 'master' },
          body: credential,
        } as never,
      )) as Schemas.ValidatePublicKeyResponse

      setIsSuccess(true)
      toast.success(t('configure_passkey.success'))

      setTimeout(() => continueLogin(created, navigate, realm_name), 1000)
    } catch (err) {
      console.error('Passkey registration failed:', err)
      if (err instanceof DOMException && (err.name === 'NotAllowedError' || err.name === 'InvalidStateError')) {
        toast.info(t('configure_passkey.already_exists'))
      } else {
        toast.error(t('configure_passkey.failed'))
      }
    } finally {
      setIsLoading(false)
    }
  }, [realm_name, navigate, t])

  return (
    <ConfigurePasskey
      onRegister={onRegister}
      isLoading={isLoading}
      isSuccess={isSuccess}
    />
  )
}
