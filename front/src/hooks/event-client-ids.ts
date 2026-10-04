import { isUuid } from '../api/uuid.ts'
import type { EventParties } from './event-role-ids.ts'

const NON_CLIENT_KINDS: readonly (string | null | undefined)[] = [
  'user',
  'service_account',
  'admin',
  'role',
]

const clientParty = (id: string | null | undefined, kind: string | null | undefined) =>
  id && isUuid(id) && !NON_CLIENT_KINDS.includes(kind) ? [id] : []

export function eventClientIds(events: readonly EventParties[]): string[] {
  return events.flatMap((event) => [
    ...clientParty(event.actor_id, event.actor_type),
    ...clientParty(event.target_id, event.target_type),
  ])
}
