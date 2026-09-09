import { useParams } from 'react-router'
import { RouterParams } from '@/routes/router'
import { useRealmDirectory } from '@/next/shared/use-realm-directory'
import { useWindowEvents } from './use-window-events'
import PageSessions from '../ui/page-sessions'

const SESSION_EVENTS = ['session_created', 'session_revoked'] as const

export default function PageSessionsFeature() {
  const { realm_name } = useParams<RouterParams>()
  const realm = realm_name ?? 'master'

  const directory = useRealmDirectory(realm)
  const { events, isLoading, isError, truncated, windowDays, windowLimit } = useWindowEvents(
    realm,
    SESSION_EVENTS
  )

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
