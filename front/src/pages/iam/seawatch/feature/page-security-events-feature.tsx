import { useMemo } from 'react'
import { useParams } from 'react-router'
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
import { useGetDailyActivityStats } from '@/api/compass.api'
import { useGetRealm } from '@/api/realm.api'
import { useWindowEvents } from '@/pages/ciam/activity/feature/use-window-events'
import PageSecurityEvents from '../ui/page-security-events'

const toDateParam = (date: Date) => date.toISOString().slice(0, 10)

export default function PageSecurityEventsFeature() {
  const { realm_name } = useParams<RouterParams>()
  const realm = realm_name ?? 'master'

  const listing = usePagedListing(SECURITY_EVENT_FILTER_KEYS)

  const {
    data: eventsResponse,
    isLoading,
    isError,
  } = useGetSecurityEvents({ realm, query: listing.apiQuery as SecurityEventsQuery })

  const recent = useWindowEvents(realm)
  const failures = useWindowEvents(realm, undefined, 'failure')
  const failedInView = useSecurityEventCount({
    realm,
    filter: { ...(listing.state.filters as SecurityEventsFilter), status: 'failure' },
  })

  const days = useMemo(() => {
    const to = new Date()
    const from = new Date(to)
    from.setDate(to.getDate() - (recent.windowDays - 1))
    return { from: toDateParam(from), to: toDateParam(to) }
  }, [recent.windowDays])

  const { data: realmResponse } = useGetRealm({ realm })
  const compassRealm = realmResponse?.settings?.compass_enabled ? realm : undefined

  const { data: activityResponse } = useGetDailyActivityStats({
    realm: compassRealm,
    from: days.from,
    to: days.to,
  })

  const events = useMemo(() => eventsResponse?.data ?? [], [eventsResponse])
  const known = useMemo(
    () => [...events, ...recent.events, ...failures.events],
    [events, recent.events, failures.events]
  )

  const activity = useMemo(() => activityResponse?.data ?? [], [activityResponse])
  const userIds = useMemo(() => eventUserIds(known), [known])
  const roleIds = useMemo(() => eventRoleIds(known), [known])
  const clientIds = useMemo(() => eventClientIds(known), [known])
  const directory = useRealmDirectory(realm, userIds, roleIds, clientIds)

  return (
    <PageSecurityEvents
      events={events}
      pagination={eventsResponse?.metadata}
      listing={listing}
      recent={recent}
      failures={failures}
      failedInView={failedInView.count}
      activity={activity}
      isLoading={isLoading}
      isError={isError || recent.isError}
      directory={directory}
    />
  )
}
