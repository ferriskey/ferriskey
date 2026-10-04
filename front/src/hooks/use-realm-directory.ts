import { useMemo } from 'react'
import { useUsersByIds } from '@/api/user.api'
import { useGetClients } from '@/api/client.api'
import { useGetRoles } from '@/api/role.api'
import type { Schemas } from '@/api/api.client'

export interface RealmDirectory {
  isLoading: boolean
  userLabel: (id?: string | null) => string | null
  label: (id?: string | null, type?: string | null) => string | null
}

const USER_KINDS: readonly (string | null | undefined)[] = ['user', 'service_account', 'admin']

export function eventUserIds(events: readonly Schemas.SecurityEvent[]): string[] {
  return events.flatMap((event) => [
    ...(event.actor_id && USER_KINDS.includes(event.actor_type) ? [event.actor_id] : []),
    ...(event.target_id && USER_KINDS.includes(event.target_type) ? [event.target_id] : []),
  ])
}

export function useRealmDirectory(realm: string, userIds: readonly string[]): RealmDirectory {
  const { users: resolved, isLoading: loadingUsers } = useUsersByIds({ realm, ids: userIds })
  const { data: clientsResponse, isLoading: loadingClients } = useGetClients({ realm })
  const { data: rolesResponse, isLoading: loadingRoles } = useGetRoles({ realm })

  return useMemo(() => {
    const users = new Map(resolved.map((u) => [u.id, u.username]))
    const clients = new Map(
      (clientsResponse?.data ?? []).map((c) => [c.id, c.name || c.client_id])
    )
    const roles = new Map((rolesResponse?.data ?? []).map((r) => [r.id, r.name]))

    const userLabel = (id?: string | null) => (id ? (users.get(id) ?? null) : null)

    const label = (id?: string | null, type?: string | null) => {
      if (!id) return null
      switch (type) {
        case 'user':
        case 'service_account':
          return users.get(id) ?? null
        case 'client':
          return clients.get(id) ?? null
        case 'role':
          return roles.get(id) ?? null
        default:
          return users.get(id) ?? clients.get(id) ?? roles.get(id) ?? null
      }
    }

    return {
      isLoading: loadingUsers || loadingClients || loadingRoles,
      userLabel,
      label,
    }
  }, [resolved, clientsResponse, rolesResponse, loadingUsers, loadingClients, loadingRoles])
}
