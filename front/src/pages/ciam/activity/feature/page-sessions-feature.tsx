import { useMemo } from 'react'
import { useParams } from 'react-router'
import { RouterParams } from '@/routes/router'
import { eventUserIds, useRealmDirectory } from '@/hooks/use-realm-directory'
import { eventRoleIds } from '@/hooks/event-role-ids'
import { useWindowEvents } from './use-window-events'
import PageSessions from '../ui/page-sessions'

const SESSION_EVENTS = ['session_created', 'session_revoked'] as const

export default function PageSessionsFeature() {
  const { realm_name } = useParams<RouterParams>()
  const realm = realm_name ?? 'master'

  const { events, isLoading, isError, truncated, windowDays, windowLimit } = useWindowEvents(
    realm,
    SESSION_EVENTS
  )
  const userIds = useMemo(() => eventUserIds(events), [events])
  const roleIds = useMemo(() => eventRoleIds(events), [events])
  const directory = useRealmDirectory(realm, userIds, roleIds)

  return (
    <PageSessions
      events={events}
      isLoading={isLoading}
      isError={isError}
      truncated={truncated}
      windowDays={windowDays}
      windowLimit={windowLimit}
      directory={directory}
    />
  )
}
