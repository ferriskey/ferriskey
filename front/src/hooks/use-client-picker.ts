import { useGetClient, useClientSearch, type ClientSearchField } from '@/api/client.api'
import type { Schemas } from '@/api/api.client'

export interface ClientPicking {
  clients: Schemas.Client[]
  selected?: Schemas.Client
  search: string
  onSearchChange: (search: string) => void
  loading: boolean
}

export function useClientPicker({
  realm,
  selectedId,
  field,
}: {
  realm: string
  selectedId?: string
  field?: ClientSearchField
}): ClientPicking {
  const { search, setSearch, clients, isLoading } = useClientSearch({ realm, field })
  const { data: selected } = useGetClient({ realm, clientId: selectedId })

  return {
    clients,
    selected: selected?.data,
    search,
    onSearchChange: setSearch,
    loading: isLoading,
  }
}
