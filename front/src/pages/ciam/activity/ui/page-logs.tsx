import { useMemo } from 'react'
import { MetricsBand } from '@/components/kit'
import type { ListingQuery, Metric } from '@/components/kit'
import { Schemas } from '@/api/api.client'
import type { RealmDirectory } from '@/hooks/use-realm-directory'
import { eventLabel, eventReason } from '@/pages/iam/seawatch/event-catalogue'
import { formatTimestamp } from '@/utils/format-date'
import { bucketPerDay } from '../feature/use-window-events'
import { ActivityPage, NoticeList, type Notice } from './activity-notices'
import { eventCard, eventColumns, searchEvent } from './event-journal'
import { JournalSection } from './journal-section'

import SecurityEvent = Schemas.SecurityEvent

export interface PageLogsProps {
  events: SecurityEvent[]
  isLoading: boolean
  isError: boolean
  truncated: boolean
  windowDays: number
  windowLimit: number
  listing: ListingQuery
  directory: RealmDirectory
}

const filters = [
  { key: 'all', label: 'All' },
  { key: 'failures', label: 'Failures' },
  { key: 'authentication', label: 'Authentication' },
  { key: 'credentials', label: 'Credentials' },
  { key: 'administration', label: 'Administration' },
]

const dominant = (values: (string | null | undefined)[]) => {
  const counts = new Map<string, number>()
  values.forEach((value) => {
    if (!value) return
    counts.set(value, (counts.get(value) ?? 0) + 1)
  })
  const [top] = Array.from(counts.entries()).sort((a, b) => b[1] - a[1])
  return top ?? null
}

const latestTimestamp = (events: SecurityEvent[]) => {
  const latest = events.reduce(
    (acc, event) =>
      !acc || new Date(event.timestamp).getTime() > new Date(acc).getTime()
        ? event.timestamp
        : acc,
    ''
  )
  return latest ? formatTimestamp(latest) : null
}

export default function PageLogs({
  events,
  isLoading,
  isError,
  truncated,
  windowDays,
  windowLimit,
  listing,
  directory,
}: PageLogsProps) {
  const windowLabel = `last ${windowDays} days`

  const failures = events.filter((e) => e.status === 'failure')
  const successes = events.length - failures.length
  const successRate = events.length ? Math.round((successes / events.length) * 100) : 0
  const uniqueActors = new Set(events.map((e) => e.actor_id ?? 'unknown')).size

  const topFailure = dominant(failures.map((e) => eventLabel(e)))
  const topErrorCode = dominant(failures.map((e) => eventReason(e)?.errorCode))

  const buckets = useMemo(() => bucketPerDay(events, windowDays), [events, windowDays])

  const measured = (series: (number | null)[]) =>
    !truncated && series.length > 0 ? series : undefined

  const successRatePerDay: (number | null)[] = buckets.map((bucket) => {
    if (bucket.length === 0) return null
    const failed = bucket.filter((event) => event.status === 'failure').length
    return Math.round(((bucket.length - failed) / bucket.length) * 100)
  })

  const metrics: Metric[] = [
    {
      key: 'total',
      label: 'Events',
      value: events.length.toLocaleString(),
      hint: truncated
        ? `capped, ${windowLabel}`
        : (latestTimestamp(events) ? `last ${latestTimestamp(events)}` : windowLabel),
      series: measured(buckets.map((bucket) => bucket.length)),
      tone: 'info',
    },
    {
      key: 'failures',
      label: 'Failures',
      value: failures.length.toLocaleString(),
      hint: topErrorCode
        ? topErrorCode[0]
        : topFailure
          ? topFailure[0].toLowerCase()
          : 'no failure recorded',
      series: measured(
        buckets.map((bucket) => bucket.filter((event) => event.status === 'failure').length)
      ),
      tone: 'brand',
    },
    {
      key: 'rate',
      label: 'Success rate',
      value: `${successRate}%`,
      hint: `${successes.toLocaleString()} successful`,
      series: measured(successRatePerDay),
      tone: 'success',
    },
    {
      key: 'actors',
      label: 'Distinct accounts',
      value: uniqueActors.toLocaleString(),
      hint: `over the ${windowLabel}`,
      series: measured(
        buckets.map(
          (bucket) => new Set(bucket.map((event) => event.actor_id ?? 'unknown')).size
        )
      ),
      tone: 'violet',
    },
  ]

  const notices: Notice[] = [
    ...(isError
      ? [
          {
            tone: 'error' as const,
            title: 'Event feed unavailable',
            detail: 'We could not fetch the latest events. Please try again later.',
          },
        ]
      : []),
    ...(truncated
      ? [
          {
            tone: 'warn' as const,
            title: `Capped at ${windowLimit} events`,
            detail: `The realm recorded more over the ${windowLabel}; the figures below cover the ${windowLimit} most recent only.`,
          },
        ]
      : []),
    ...(failures.length > 0
      ? [
          {
            tone: 'error' as const,
            title: `${failures.length} failed event${failures.length > 1 ? 's' : ''} over the ${windowLabel}`,
            detail: [
              topFailure ? `${topFailure[0]} accounts for ${topFailure[1]}` : null,
              topErrorCode ? `mostly ${topErrorCode[0]}` : null,
            ]
              .filter(Boolean)
              .join(' · '),
          },
        ]
      : []),
  ]

  const columns = eventColumns(directory)

  const filtered = useMemo(() => {
    const out =
      listing.filter === 'failures' ? events.filter((e) => e.status === 'failure') : events
    const needle = listing.draft.trim().toLowerCase()
    if (!needle) return out
    return out.filter((e) => searchEvent(e).toLowerCase().includes(needle))
  }, [events, listing.filter, listing.draft])

  return (
    <ActivityPage
      title='Logs & events'
      description={`Every authentication, credential and administrative event recorded in this realm over the ${windowLabel}, most recent first.`}
    >
      <NoticeList notices={notices} />

      <MetricsBand metrics={metrics} />

      <JournalSection
        title='Event feed'
        description='Event families are applied by the server; the search box narrows what was loaded above.'
        rows={filtered}
        total={events.length}
        columns={[
          columns.event,
          columns.outcome,
          columns.actor,
          columns.target,
          columns.origin,
          columns.when,
        ]}
        card={eventCard(directory)}
        getKey={(e) => e.id}
        loading={isLoading}
        filters={filters}
        filter={listing.filter}
        onFilter={listing.setFilter}
        query={listing.draft}
        onQuery={listing.setDraft}
        searchPlaceholder='Search by event, account, IP…'
        aggregates={{
          event_type: `${events.length} event${events.length !== 1 ? 's' : ''}`,
          status: `${failures.length} failed`,
          actor: `${uniqueActors} account${uniqueActors !== 1 ? 's' : ''}`,
        }}
        emptyLabel='No event'
        emptyHint={`This feed fills as customers sign in, reset credentials and as administrators change the realm. Nothing was recorded over the ${windowLabel}.`}
      />
    </ActivityPage>
  )
}
