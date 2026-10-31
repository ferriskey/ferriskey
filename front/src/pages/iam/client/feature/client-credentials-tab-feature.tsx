import { useState } from 'react'
import {
  useGetClientSecret,
  useRevokePreviousClientSecret,
  useRotateClientSecret,
} from '@/api/client.api'
import type { Schemas } from '@/api/api.client'
import ClientCredentialsTab from '../ui/client-credentials-tab'

type Client = Schemas.Client

export interface ClientCredentialsTabFeatureProps {
  client: Client
  realm: string
}

export default function ClientCredentialsTabFeature({
  client,
  realm,
}: ClientCredentialsTabFeatureProps) {
  const [revealed, setRevealed] = useState(false)

  const { data, error, isFetching } = useGetClientSecret({
    realm,
    clientId: client.id,
    enabled: revealed,
  })

  const { mutate: rotateSecret, isPending: rotating } = useRotateClientSecret()
  const { mutate: revokePrevious, isPending: revoking } = useRevokePreviousClientSecret()
  const path = { realm_name: realm, client_id: client.id }

  const status = (error as { status?: number } | null)?.status
  const forbidden = revealed && status === 403
  const failed = revealed && Boolean(error) && !forbidden

  return (
    <ClientCredentialsTab
      client={client}
      revealed={revealed}
      secret={revealed ? (data?.client_secret ?? null) : null}
      isFetching={isFetching}
      forbidden={forbidden}
      failed={failed}
      previousSecret={revealed ? (data?.previous_secret?.client_secret ?? null) : null}
      rotating={rotating}
      revoking={revoking}
      onToggleReveal={() => setRevealed((r) => !r)}
      onRotate={(gracePeriodSecs) =>
        rotateSecret(
          { path, body: { grace_period_secs: gracePeriodSecs } },
          { onSuccess: () => setRevealed(true) }
        )
      }
      onRevokePrevious={() => revokePrevious({ path })}
    />
  )
}
