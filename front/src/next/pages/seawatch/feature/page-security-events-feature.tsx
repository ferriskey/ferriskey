import { useMemo } from 'react'
import { useQuery } from '@tanstack/react-query'
import { useParams } from 'react-router'
import { RouterParams } from '@/routes/router'
import PageSecurityEvents from '../ui/page-security-events'

const WINDOW_DAYS = 7
const WINDOW_LIMIT = 500

export default function PageSecurityEventsFeature() {
  const { realm_name } = useParams<RouterParams>()
  const realm = realm_name ?? 'master'

  const range = useMemo(() => {
    const to = new Date()
    const from = new Date(to.getTime() - WINDOW_DAYS * 24 * 60 * 60 * 1000)
    return { from: from.toISOString(), to: to.toISOString() }
  }, [])

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

  return (
    <PageSecurityEvents
      events={events}
      isLoading={isLoading}
      isError={isError}
      windowDays={WINDOW_DAYS}
      windowLimit={WINDOW_LIMIT}
      truncated={events.length >= WINDOW_LIMIT}
    />
  )
}
