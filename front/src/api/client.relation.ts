import { useParams } from 'react-router'
import type { RelationOption, RelationSource } from '@/components/kit'
import { RouterParams } from '@/routes/router'
import type { Schemas } from './api.client'
import { useGetClients } from './client.api'

const CLIENT_OPTION_LIMIT = 20

const toOption = (client: Schemas.Client): RelationOption => ({
  id: client.id,
  label: client.name || client.client_id,
  sublabel: client.client_id,
})

function useRealmClients() {
  const { realm_name } = useParams<RouterParams>()
  return useGetClients({ realm: realm_name ?? 'master' })
}

function useClientOptions(search: string) {
  const { data, isLoading } = useRealmClients()
  const needle = search.trim().toLowerCase()
  const options = (data?.data ?? [])
    .filter(
      (client) =>
        !needle ||
        client.name.toLowerCase().includes(needle) ||
        client.client_id.toLowerCase().includes(needle)
    )
    .slice(0, CLIENT_OPTION_LIMIT)
    .map(toOption)
  return { options, loading: isLoading }
}

function useSelectedClient(id: string | undefined) {
  const { data } = useRealmClients()
  const client = id ? data?.data.find((candidate) => candidate.id === id) : undefined
  return client ? toOption(client) : undefined
}

export const clientRelationSource: RelationSource = {
  useOptions: useClientOptions,
  useSelected: useSelectedClient,
}
