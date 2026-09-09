import { useMemo } from 'react'
import { useQuery } from '@tanstack/react-query'
import { useParams } from 'react-router'
import { RouterParams } from '@/routes/router'
import { useListingQuery } from '@/components/kit'
import { useRealmDirectory } from '@/next/shared/use-realm-directory'
import { useGetDailyActivityStats } from '@/api/compass.api'
import { useGetRealm } from '@/api/realm.api'
import { eventFamilies } from '../event-catalogue'
import PageSecurityEvents from '../ui/page-security-events'

const WINDOW_DAYS = 7
const WINDOW_LIMIT = 500

const toDateParam = (date: Date) => date.toISOString().slice(0, 10)

export default function PageSecurityEventsFeature() {
  const { realm_name } = useParams<RouterParams>()
  const realm = realm_name ?? 'master'

  const range = useMemo(() => {
    const to = new Date()
    const from = new Date(to.getTime() - WINDOW_DAYS * 24 * 60 * 60 * 1000)
    const fromDay = new Date(to)
    fromDay.setDate(to.getDate() - (WINDOW_DAYS - 1))
    return {
      from: from.toISOString(),
      to: to.toISOString(),
      fromDay: toDateParam(fromDay),
      toDay: toDateParam(to),
    }
  }, [])

  const listing = useListingQuery()
  const directory = useRealmDirectory(realm)
  const family = eventFamilies[listing.filter]

  const { data: realmResponse } = useGetRealm({ realm })
  const compassRealm = realmResponse?.settings?.compass_enabled ? realm : undefined

  const { data: activityResponse } = useGetDailyActivityStats({
    realm: compassRealm,
    from: range.fromDay,
    to: range.toDay,
  })

  const {
    data: eventsResponse,
    isLoading,
    isError,
  } = useQuery({
    ...window.tanstackApi.get(
      '/realms/{realm_name}/seawatch/v1/security-events',
      {
        path: { realm_name: realm },
        query: {
          from_timestamp: range.from,
          to_timestamp: range.to,
          limit: WINDOW_LIMIT,
          event_types: family ? family.join(',') : undefined,
        },
      }
    ).queryOptions,
    enabled: Boolean(realm_name),
  })

  const events = useMemo(
    () =>
      [...(eventsResponse?.data ?? [])].sort(
        (a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime()
      ),
    [eventsResponse]
  )

  const activity = useMemo(() => activityResponse?.data ?? [], [activityResponse])

  return (
    <PageSecurityEvents
      events={events}
      activity={activity}
      isLoading={isLoading}
      isError={isError}
      windowDays={WINDOW_DAYS}
      windowLimit={WINDOW_LIMIT}
      truncated={events.length >= WINDOW_LIMIT}
      listing={listing}
      directory={directory}
    />
  )
}
