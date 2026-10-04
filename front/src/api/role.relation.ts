import { useParams } from 'react-router'
import type { RelationOption, RelationSource } from '@/components/kit'
import { RouterParams } from '@/routes/router'
import type { Schemas } from './api.client'
import { useGetRoles } from './role.api'

const toOption = (role: Schemas.Role): RelationOption => ({
  id: role.id,
  label: role.name,
  sublabel: role.description ?? undefined,
})

function useRealmRoles() {
  const { realm_name } = useParams<RouterParams>()
  return useGetRoles({ realm: realm_name ?? 'master' })
}

function useRoleOptions(search: string) {
  const { data, isLoading } = useRealmRoles()
  const needle = search.trim().toLowerCase()
  const options = (data?.data ?? [])
    .filter((role) => !needle || role.name.toLowerCase().includes(needle))
    .map(toOption)
  return { options, loading: isLoading }
}

function useSelectedRole(id: string | undefined) {
  const { data } = useRealmRoles()
  const role = id ? data?.data.find((candidate) => candidate.id === id) : undefined
  return role ? toOption(role) : undefined
}

export const roleRelationSource: RelationSource = {
  useOptions: useRoleOptions,
  useSelected: useSelectedRole,
}
