import { useMemo } from 'react'
import type { PickableEntity } from '@/components/kit'
import { useUserSearch, useUsersByIds, type UsersFilter } from '@/api/user.api'
import type { Schemas } from '@/api/api.client'

export const toPickableUser = (user: Schemas.User): PickableEntity => ({
  id: user.id,
  label: user.username,
  sublabel: user.email ?? undefined,
})

export function useUserPicker({
  realm,
  selectedIds,
  filter,
  toItem = toPickableUser,
}: {
  realm: string
  selectedIds: readonly string[]
  filter?: UsersFilter
  toItem?: (user: Schemas.User) => PickableEntity
}) {
  const { users: selected } = useUsersByIds({ realm, ids: selectedIds })
  const { setSearch, users: found, isLoading } = useUserSearch({ realm, filter })

  const items = useMemo(() => {
    const byId = new Map<string, PickableEntity>()
    for (const user of [...selected, ...found]) byId.set(user.id, toItem(user))
    return [...byId.values()]
  }, [selected, found, toItem])

  return { items, onSearchChange: setSearch, loading: isLoading }
}
