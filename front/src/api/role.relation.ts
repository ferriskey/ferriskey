import { useParams } from 'react-router'
import type { RelationOption, RelationSource } from '@/components/kit'
import { RouterParams } from '@/routes/router'
import type { Schemas } from './api.client'
import { ROLE_SEARCH_LIMIT, useGetRole, useGetRoles } from './role.api'

const toOption = (role: Schemas.Role): RelationOption => ({
  id: role.id,
  label: role.name,
  sublabel: role.description ?? undefined,
})

function useRealm() {
  const { realm_name } = useParams<RouterParams>()
  return realm_name ?? 'master'
}

function useRoleOptions(search: string) {
  const realm = useRealm()
  const { data, isLoading } = useGetRoles({
    realm,
    query: { name: search.trim() || undefined, limit: ROLE_SEARCH_LIMIT },
  })
  return { options: (data?.data ?? []).map(toOption), loading: isLoading }
}

function useSelectedRole(id: string | undefined) {
  const realm = useRealm()
  const { data } = useGetRole({ realm, roleId: id })
  return data?.data ? toOption(data.data) : undefined
}

export const roleRelationSource: RelationSource = {
  useOptions: useRoleOptions,
  useSelected: useSelectedRole,
}
