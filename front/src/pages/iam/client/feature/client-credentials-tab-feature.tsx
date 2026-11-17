import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'
import { useGetClientSecret, useUpdateClient } from '@/api/client.api'
import type { Schemas } from '@/api/api.client'
import { apiErrorMessage } from '@/lib/api-error'
import ClientCredentialsTab from '../ui/client-credentials-tab'
import type { ClientAuthenticationDraft, ConfidentialAuthMethod } from '../client-authentication'

type Client = Schemas.Client

export interface ClientCredentialsTabFeatureProps {
  client: Client
  realm: string
}

interface Draft extends ClientAuthenticationDraft {
  key: string
}

const pristineDraft = (client: Client): Draft => ({
  key: `${client.id}:${client.updated_at}`,
  method:
    client.token_endpoint_auth_method === 'none'
      ? 'client_secret_basic'
      : (client.token_endpoint_auth_method as ConfidentialAuthMethod),
  keySource: client.jwks ? 'jwks' : 'jwks_uri',
  jwksUri: client.jwks_uri ?? '',
  jwks: client.jwks ? JSON.stringify(client.jwks, null, 2) : '',
})

const parseKeySet = (raw: string): Schemas.ClientJwks | undefined => {
  try {
    const parsed: unknown = JSON.parse(raw)
    return parsed && typeof parsed === 'object' && !Array.isArray(parsed)
      ? (parsed as Schemas.ClientJwks)
      : undefined
  } catch {
    return undefined
  }
}

const usesSecret = (method: Schemas.TokenEndpointAuthMethod) => method !== 'private_key_jwt'

export default function ClientCredentialsTabFeature({
  client,
  realm,
}: ClientCredentialsTabFeatureProps) {
  const { t } = useTranslation('client')
  const [revealed, setRevealed] = useState(false)

  const { data, error, isFetching } = useGetClientSecret({
    realm,
    clientId: client.id,
    enabled: revealed,
  })
  const { mutate: updateClient, isPending: saving } = useUpdateClient()

  const pristine = pristineDraft(client)
  const [draft, setDraft] = useState<Draft>(pristine)
  if (draft.key !== pristine.key) setDraft(pristine)
  const current = draft.key === pristine.key ? draft : pristine

  const status = (error as { status?: number } | null)?.status
  const forbidden = revealed && status === 403
  const failed = revealed && Boolean(error) && !forbidden

  const needsKeys = current.method === 'private_key_jwt'
  const inlineKeys = needsKeys && current.keySource === 'jwks'
  const parsedKeys = inlineKeys ? parseKeySet(current.jwks) : undefined

  let authError: string | undefined
  if (needsKeys && !inlineKeys && current.jwksUri.trim() === '') {
    authError = t('credentials.authentication.keys.uri_required')
  } else if (inlineKeys && !parsedKeys) {
    authError = t('credentials.authentication.keys.invalid_json')
  }

  const dirty =
    current.method !== pristine.method ||
    (needsKeys &&
      (current.keySource !== pristine.keySource ||
        (inlineKeys ? current.jwks !== pristine.jwks : current.jwksUri !== pristine.jwksUri)))

  const save = () => {
    if (authError) return

    updateClient(
      {
        body: {
          token_endpoint_auth_method: current.method,
          jwks: inlineKeys ? (parsedKeys ?? null) : null,
          jwks_uri: needsKeys && !inlineKeys ? current.jwksUri.trim() : null,
        },
        path: { client_id: client.id, realm_name: realm },
      },
      { onError: (updateError) => toast.error(apiErrorMessage(updateError)) }
    )
  }

  return (
    <ClientCredentialsTab
      client={client}
      revealed={revealed}
      secret={revealed ? (data?.client_secret ?? null) : null}
      isFetching={isFetching}
      forbidden={forbidden}
      failed={failed}
      onToggleReveal={() => setRevealed((r) => !r)}
      showSecret={client.public_client || usesSecret(client.token_endpoint_auth_method)}
      authentication={
        client.public_client || client.protocol !== 'openid-connect'
          ? undefined
          : {
              draft: current,
              error: authError,
              dirty,
              saving,
              onChange: (patch) => setDraft((d) => ({ ...d, ...patch })),
              onDiscard: () => setDraft(pristine),
              onSave: save,
            }
      }
    />
  )
}
