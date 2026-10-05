import { useMemo } from 'react'
import { useParams } from 'react-router'
import { useTranslation } from 'react-i18next'
import { RouterParams } from '@/routes/router'
import { usePagedListing } from '@/components/kit'
import {
  SECURITY_EVENT_FILTER_KEYS,
  useGetSecurityEvents,
  useSecurityEventCount,
  type SecurityEventsFilter,
  type SecurityEventsQuery,
} from '@/api/sea-watch.api'
import { eventUserIds, useRealmDirectory } from '@/hooks/use-realm-directory'
import { eventRoleIds } from '@/hooks/event-role-ids'
import { eventClientIds } from '@/hooks/event-client-ids'
import { useWindowEvents } from './use-window-events'
import PageLogs from '../ui/page-logs'

const SEAWATCH_NAMESPACE = 'seawatch'

export default function PageLogsFeature() {
  useTranslation(SEAWATCH_NAMESPACE)

  const { realm_name } = useParams<RouterParams>()
  const realm = realm_name ?? 'master'

  const listing = usePagedListing(SECURITY_EVENT_FILTER_KEYS)
  const {
    data: eventsResponse,
    isLoading,
    isError,
  } = useGetSecurityEvents({ realm, query: listing.apiQuery as SecurityEventsQuery, keepPrevious: true })

  const recent = useWindowEvents(realm)
  const failures = useWindowEvents(realm, undefined, 'failure')
  const failedInView = useSecurityEventCount({
    realm,
    filter: { ...(listing.state.filters as SecurityEventsFilter), status: 'failure' },
  })

  const events = useMemo(() => eventsResponse?.data ?? [], [eventsResponse])
  const known = useMemo(
    () => [...events, ...recent.events, ...failures.events],
    [events, recent.events, failures.events]
  )
  const userIds = useMemo(() => eventUserIds(known), [known])
  const roleIds = useMemo(() => eventRoleIds(known), [known])
  const clientIds = useMemo(() => eventClientIds(known), [known])
  const directory = useRealmDirectory(realm, userIds, roleIds, clientIds)

  return (
    <PageLogs
      events={events}
      pagination={eventsResponse?.metadata}
      listing={listing}
      recent={recent}
      failures={failures}
      failedInView={failedInView.count}
      isLoading={isLoading}
      isError={isError || recent.isError}
      directory={directory}
    />
  )
}
