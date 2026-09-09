import { useState } from 'react'
import { useGetClientSecret } from '@/api/client.api'
import { Schemas } from '@/api/api.client'
import ClientCredentialsTab from '../ui/client-credentials-tab'

import Client = Schemas.Client

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
      onToggleReveal={() => setRevealed((r) => !r)}
    />
  )
}
