import { useParams } from 'react-router'
import type { RelationOption, RelationSource } from '@/components/kit'
import { RouterParams } from '@/routes/router'
import type { Schemas } from './api.client'
import { CLIENT_SEARCH_LIMIT, useGetClient, useGetClients } from './client.api'

const toOption = (client: Schemas.Client): RelationOption => ({
  id: client.id,
  label: client.name || client.client_id,
  sublabel: client.client_id,
})

function useRealm() {
  const { realm_name } = useParams<RouterParams>()
  return realm_name ?? 'master'
}

function useClientOptions(search: string) {
  const { data, isLoading } = useGetClients({
    realm: useRealm(),
    query: { name: search.trim() || undefined, limit: CLIENT_SEARCH_LIMIT },
  })
  return { options: (data?.data ?? []).map(toOption), loading: isLoading }
}

function useSelectedClient(id: string | undefined) {
  const { data } = useGetClient({ realm: useRealm(), clientId: id })
  return data?.data ? toOption(data.data) : undefined
}

export const clientRelationSource: RelationSource = {
  useOptions: useClientOptions,
  useSelected: useSelectedClient,
}
