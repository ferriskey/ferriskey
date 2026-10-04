import { useParams } from 'react-router'
import { useTranslation } from 'react-i18next'
import { CLIENT_SEARCH_LIMIT, useGetClients } from '@/api/client.api'
import { ROLE_SEARCH_LIMIT, useGetRoles } from '@/api/role.api'
import { RouterParams } from '@/routes/router'
import type { Schemas } from '@/api/api.client'
import type { MapperEntityOptions } from '../ui/mapper-config-fields'
import type { MapperEntityOption } from '../ui/mapper-entity-field'

const LOOKUP_LIMIT = 100

function useRealm() {
  const { realm_name } = useParams<RouterParams>()
  return realm_name ?? 'master'
}

const toClientOption = (client: Schemas.Client): MapperEntityOption => ({
  value: client.client_id,
  label: client.name,
  sublabel: client.client_id,
})

function useClientOptions(search: string) {
  const { data, isLoading } = useGetClients({
    realm: useRealm(),
    query: {
      name: search.trim() || undefined,
      order_by: 'name',
      order: 'asc',
      limit: CLIENT_SEARCH_LIMIT,
    },
  })
  return { options: (data?.data ?? []).map(toClientOption), loading: isLoading }
}

function useSelectedClient(value: string) {
  const { data, isLoading } = useGetClients({
    realm: useRealm(),
    query: { client_id: value, limit: LOOKUP_LIMIT },
    enabled: value !== '',
  })
  const client = data?.data.find((candidate) => candidate.client_id === value)
  return { option: client ? toClientOption(client) : undefined, loading: value !== '' && isLoading }
}

const roleValue = (role: Schemas.Role) => {
  const clientId = role.client?.client_id
  return clientId ? `${clientId}.${role.name}` : role.name
}

function useRoleOption() {
  const { t } = useTranslation('client-scope')
  return (role: Schemas.Role): MapperEntityOption => ({
    value: roleValue(role),
    label: role.name,
    sublabel: role.client?.client_id ? roleValue(role) : t('mapper_entity.role.realm_role'),
  })
}

function useRoleOptions(search: string) {
  const toOption = useRoleOption()
  const { data, isLoading } = useGetRoles({
    realm: useRealm(),
    query: {
      search: search.trim() || undefined,
      order_by: 'name',
      order: 'asc',
      limit: ROLE_SEARCH_LIMIT,
    },
  })
  return { options: (data?.data ?? []).map(toOption), loading: isLoading }
}

function useSelectedRole(value: string) {
  const toOption = useRoleOption()
  const needle = value.slice(value.lastIndexOf('.') + 1)
  const { data, isLoading } = useGetRoles({
    realm: useRealm(),
    query: { name: needle, limit: LOOKUP_LIMIT },
    enabled: needle !== '',
  })
  const role = data?.data.find((candidate) => roleValue(candidate) === value)
  return { option: role ? toOption(role) : undefined, loading: needle !== '' && isLoading }
}

export const MAPPER_ENTITY_SOURCES: MapperEntityOptions = {
  client: { useOptions: useClientOptions, useSelected: useSelectedClient },
  role: { useOptions: useRoleOptions, useSelected: useSelectedRole },
}
