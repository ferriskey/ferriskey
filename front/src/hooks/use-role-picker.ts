import { useMemo } from 'react'
import type { PickableEntity } from '@/components/kit'
import { useRoleSearch, useRolesByIds, type RolesFilter } from '@/api/role.api'
import type { Schemas } from '@/api/api.client'

export const toPickableRole = (role: Schemas.Role): PickableEntity => ({
  id: role.id,
  label: role.name,
  sublabel: role.description ?? undefined,
})

export function useRolePicker({
  realm,
  selectedIds,
  filter,
}: {
  realm: string
  selectedIds: readonly string[]
  filter?: RolesFilter
}) {
  const { roles: selected } = useRolesByIds({ realm, ids: selectedIds })
  const { setSearch, roles: found, isLoading } = useRoleSearch({ realm, filter })

  const roles = useMemo(() => {
    const byId = new Map<string, Schemas.Role>()
    for (const role of [...selected, ...found]) byId.set(role.id, role)
    return [...byId.values()]
  }, [selected, found])

  const items = useMemo(() => roles.map(toPickableRole), [roles])

  return { roles, items, onSearchChange: setSearch, loading: isLoading }
}
