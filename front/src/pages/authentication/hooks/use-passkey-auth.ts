import { useCallback, useEffect, useRef, useState } from 'react'
import { toast } from 'sonner'
import {
  usePasskeyAuthenticateMutation,
  usePasskeyRequestOptionsMutation,
} from '@/api/passkey.api'
import {
  isConditionalMediationAvailable,
  isWebAuthnAvailable,
  startAuthentication,
  startConditionalAuthentication,
  type PublicKeyCredentialRequestOptionsJSON,
} from '@/lib/webauthn'

type Options = {
  realm_name: string | undefined
  enabled: boolean
  isAuthInitiated: boolean
}

export function usePasskeyAuth({ realm_name, enabled, isAuthInitiated }: Options) {
  const { mutateAsync: requestPasskeyOptionsAsync, mutate: requestPasskeyOptions } =
    usePasskeyRequestOptionsMutation()
  const { mutateAsync: authenticatePasskeyAsync, mutate: authenticatePasskey } =
    usePasskeyAuthenticateMutation()

  const [isPasskeyLoading, setIsPasskeyLoading] = useState(false)
  const [conditionalUIVersion, setConditionalUIVersion] = useState(0)
  const conditionalAbortRef = useRef<AbortController | null>(null)

  // Conditional UI: autofill passkeys in the username field (Apple Passkeys, Chrome, etc.)
  useEffect(() => {
    if (!enabled || !isAuthInitiated) return

    let aborted = false
    const abortController = new AbortController()
    conditionalAbortRef.current = abortController

    const startConditionalUI = async () => {
      const available = await isConditionalMediationAvailable()
      if (!available || aborted) return

      try {
        const response = await requestPasskeyOptionsAsync({
          realm: realm_name,
          data: {},
        })

        if (aborted) return

        const assertion = await startConditionalAuthentication(
          response.publicKey as PublicKeyCredentialRequestOptionsJSON,
          abortController.signal
        )

        if (!assertion || aborted) return

        const result = await authenticatePasskeyAsync({
          realm: realm_name,
          data: assertion,
        })

        if (result.login_url) {
          window.location.href = result.login_url
        }
      } catch {
        // Conditional UI was aborted or failed silently — this is expected
      }
    }

    startConditionalUI()

    return () => {
      aborted = true
      abortController.abort()
      conditionalAbortRef.current = null
    }
  }, [
    enabled,
    isAuthInitiated,
    realm_name,
    requestPasskeyOptionsAsync,
    authenticatePasskeyAsync,
    conditionalUIVersion,
  ])

  const onPasskeyLogin = useCallback(() => {
    if (!isWebAuthnAvailable()) {
      toast.error('WebAuthn is not supported in this browser')
      return
    }

    // Abort any ongoing conditional UI request
    conditionalAbortRef.current?.abort()
    conditionalAbortRef.current = null

    setIsPasskeyLoading(true)

    requestPasskeyOptions(
      { realm: realm_name, data: {} },
      {
        onSuccess: async (response) => {
          try {
            const assertion = await startAuthentication(
              response.publicKey as PublicKeyCredentialRequestOptionsJSON
            )
            authenticatePasskey(
              { realm: realm_name, data: assertion },
              {
                onSuccess: (result) => {
                  if (result.login_url) {
                    window.location.href = result.login_url
                  }
                },
                onError: () => {
                  toast.error('Passkey authentication failed')
                  setIsPasskeyLoading(false)
                  setConditionalUIVersion((v) => v + 1)
                },
              }
            )
          } catch {
            setIsPasskeyLoading(false)
          }
        },
        onError: () => {
          toast.error('Failed to start passkey authentication')
          setIsPasskeyLoading(false)
          setConditionalUIVersion((v) => v + 1)
        },
      }
    )
  }, [realm_name, requestPasskeyOptions, authenticatePasskey])

  return {
    onPasskeyLogin,
    isPasskeyLoading,
  }
}
