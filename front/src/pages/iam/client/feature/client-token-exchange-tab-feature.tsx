import { useState } from 'react'
import { useGetClients } from '@/api/client.api'
import {
  useCreateTokenExchangePolicy,
  useDeleteTokenExchangePolicy,
  useGetTokenExchangePolicies,
} from '@/api/token-exchange-policy.api'
import { Schemas } from '@/api/api.client'
import type { TokenExchangePolicySchema } from '../schemas/token-exchange-policy.schema'
import ClientTokenExchangeTab from '../ui/client-token-exchange-tab'

import Client = Schemas.Client
import TokenExchangePolicy = Schemas.TokenExchangePolicy

export interface ClientTokenExchangeTabFeatureProps {
  client: Client
  realm: string
}

export default function ClientTokenExchangeTabFeature({
  client,
  realm,
}: ClientTokenExchangeTabFeatureProps) {
  const [createOpen, setCreateOpen] = useState(false)
  const [pendingDelete, setPendingDelete] = useState<TokenExchangePolicy | null>(null)

  const {
    data: policies = [],
    isLoading,
    isError,
  } = useGetTokenExchangePolicies({ realm, clientId: client.id })
  const { data: clientsResponse } = useGetClients({ realm })
  const { mutateAsync: createPolicy, isPending: isCreating } = useCreateTokenExchangePolicy()
  const { mutateAsync: deletePolicy, isPending: isDeleting } = useDeleteTokenExchangePolicy()

  const audiences = (clientsResponse?.data ?? []).filter((candidate) => candidate.id !== client.id)

  const handleCreate = async (values: TokenExchangePolicySchema) => {
    const created = await createPolicy({
      path: { realm_name: realm, client_id: client.id },
      body: {
        target_audience: values.targetAudience,
        allowed_scopes: values.allowedScopes.length > 0 ? values.allowedScopes : null,
        allow_impersonation: values.allowImpersonation,
        allow_delegation: values.allowDelegation,
      },
    })
      .then(() => true)
      .catch(() => false)

    if (created) setCreateOpen(false)
  }

  const handleDelete = async () => {
    if (!pendingDelete) return

    await deletePolicy({
      path: { realm_name: realm, client_id: client.id, policy_id: pendingDelete.id },
    }).catch(() => undefined)
    setPendingDelete(null)
  }

  return (
    <ClientTokenExchangeTab
      client={client}
      policies={policies}
      audiences={audiences}
      isLoading={isLoading}
      isError={isError}
      isCreating={isCreating}
      isDeleting={isDeleting}
      createOpen={createOpen}
      pendingDelete={pendingDelete}
      onCreateOpenChange={setCreateOpen}
      onCreate={(values) => void handleCreate(values)}
      onRequestDelete={setPendingDelete}
      onCancelDelete={() => setPendingDelete(null)}
      onConfirmDelete={() => void handleDelete()}
    />
  )
}
