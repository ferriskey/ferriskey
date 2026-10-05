export interface EventParties {
  actor_id?: string | null
  actor_type?: string | null
  target_id?: string | null
  target_type?: string | null
}

const NON_ROLE_KINDS: readonly (string | null | undefined)[] = [
  'user',
  'service_account',
  'admin',
  'client',
]

export function eventRoleIds(events: readonly EventParties[]): string[] {
  return events.flatMap((event) => [
    ...(event.actor_id && !NON_ROLE_KINDS.includes(event.actor_type) ? [event.actor_id] : []),
    ...(event.target_id && !NON_ROLE_KINDS.includes(event.target_type) ? [event.target_id] : []),
  ])
}
