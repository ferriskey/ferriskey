import { useMemo } from 'react'
import { useParams } from 'react-router'
import { RouterParams } from '@/routes/router'
import { eventUserIds, useRealmDirectory } from '@/hooks/use-realm-directory'
import { eventRoleIds } from '@/hooks/event-role-ids'
import { eventClientIds } from '@/hooks/event-client-ids'
import { useEventJournal } from './use-event-journal'
import { useWindowCount, useWindowEvents } from './use-window-events'
import PageSessions from '../ui/page-sessions'

const SESSION_EVENTS = ['session_created', 'session_revoked'] as const


export default function PageSessionsFeature() {
  const { realm_name } = useParams<RouterParams>()
  const realm = realm_name ?? 'master'

  const { events, total, range, isLoading, isError, truncated, windowDays, windowLimit } =
    useWindowEvents(realm, SESSION_EVENTS)
  const opened = useWindowCount(realm, range, { event_types: 'session_created' })
  const revoked = useWindowCount(realm, range, { event_types: 'session_revoked' })
  const journal = useEventJournal(realm, SESSION_EVENTS)
  const known = useMemo(() => [...events, ...journal.events], [events, journal.events])
  const userIds = useMemo(() => eventUserIds(known), [known])
  const roleIds = useMemo(() => eventRoleIds(known), [known])
  const clientIds = useMemo(() => eventClientIds(known), [known])
  const directory = useRealmDirectory(realm, userIds, roleIds, clientIds)

  return (
    <PageSessions
      events={events}
      counts={{ opened, revoked, total }}
      journal={journal}
      isLoading={isLoading}
      isError={isError || journal.isError}
      truncated={truncated}
      windowDays={windowDays}
      windowLimit={windowLimit}
      directory={directory}
    />
  )
}
