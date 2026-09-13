import { useMemo, useState } from 'react'
import { MetricsBand } from '@/components/kit'
import type { Metric } from '@/components/kit'
import { Schemas } from '@/api/api.client'
import type { RealmDirectory } from '@/hooks/use-realm-directory'
import { formatTimestamp } from '@/utils/format-date'
import { bucketPerDay } from '../feature/use-window-events'
import { ActivityPage, NoticeList, type Notice } from './activity-notices'
import { eventCard, eventColumns, searchEvent } from './event-journal'
import { JournalSection } from './journal-section'

import SecurityEvent = Schemas.SecurityEvent

export interface PageSessionsProps {
  events: SecurityEvent[]
  isLoading: boolean
  isError: boolean
  truncated: boolean
  windowDays: number
  windowLimit: number
  directory: RealmDirectory
}

const filters = [
  { key: 'all', label: 'All' },
  { key: 'opened', label: 'Opened' },
  { key: 'revoked', label: 'Revoked' },
]

export default function PageSessions({
  events,
  isLoading,
  isError,
  truncated,
  windowDays,
  windowLimit,
  directory,
}: PageSessionsProps) {
  const [filter, setFilter] = useState('all')
  const [query, setQuery] = useState('')

  const windowLabel = `last ${windowDays} days`

  const opened = events.filter((e) => e.event_type === 'session_created')
  const revoked = events.filter((e) => e.event_type === 'session_revoked')
  const accounts = new Set(events.map((e) => e.actor_id ?? 'unknown')).size
  const origins = new Set(events.map((e) => e.ip_address).filter(Boolean)).size

  const buckets = useMemo(() => bucketPerDay(events, windowDays), [events, windowDays])

  const measured = (series: number[]) => (!truncated && series.length > 0 ? series : undefined)

  const metrics: Metric[] = [
    {
      key: 'opened',
      label: 'Sessions opened',
      value: opened.length.toLocaleString(),
      hint: `over the ${windowLabel}`,
      series: measured(
        buckets.map((b) => b.filter((e) => e.event_type === 'session_created').length)
      ),
      tone: 'success',
    },
    {
      key: 'revoked',
      label: 'Sessions revoked',
      value: revoked.length.toLocaleString(),
      hint: revoked.length === 0 ? 'none revoked' : `over the ${windowLabel}`,
      series: measured(
        buckets.map((b) => b.filter((e) => e.event_type === 'session_revoked').length)
      ),
      tone: 'brand',
    },
    {
      key: 'accounts',
      label: 'Distinct accounts',
      value: accounts.toLocaleString(),
      hint: `over the ${windowLabel}`,
      series: measured(
        buckets.map((b) => new Set(b.map((e) => e.actor_id ?? 'unknown')).size)
      ),
      tone: 'violet',
    },
    {
      key: 'origins',
      label: 'Distinct origins',
      value: origins.toLocaleString(),
      hint: origins === 0 ? 'no IP address recorded' : 'IP addresses seen',
      series: measured(
        buckets.map((b) => new Set(b.map((e) => e.ip_address).filter(Boolean)).size)
      ),
      tone: 'info',
    },
  ]

  const notices: Notice[] = [
    {
      tone: 'note',
      title: 'No realm-wide list of active devices exists',
      detail:
        'The API lists and revokes sessions one account at a time, so the devices currently signed in are shown on each account. What is recorded for the whole realm is the journal below.',
    },
    ...(isError
      ? [
          {
            tone: 'error' as const,
            title: 'Session journal unavailable',
            detail: 'We could not fetch the session events of this realm. Please try again later.',
          },
        ]
      : []),
    ...(truncated
      ? [
          {
            tone: 'warn' as const,
            title: `Capped at ${windowLimit} events`,
            detail: `The realm recorded more over the ${windowLabel}; the figures above cover the ${windowLimit} most recent only.`,
          },
        ]
      : []),
  ]

  const columns = eventColumns(directory)

  const filtered = useMemo(() => {
    const byFilter =
      filter === 'opened'
        ? events.filter((e) => e.event_type === 'session_created')
        : filter === 'revoked'
          ? events.filter((e) => e.event_type === 'session_revoked')
          : events
    const needle = query.trim().toLowerCase()
    if (!needle) return byFilter
    return byFilter.filter((e) => searchEvent(e).toLowerCase().includes(needle))
  }, [events, filter, query])

  const lastOpened = opened[0]

  return (
    <ActivityPage
      title='Sessions'
      description={`Sessions opened and revoked in this realm over the ${windowLabel}${
        lastOpened ? `, most recently on ${formatTimestamp(lastOpened.timestamp)}` : ''
      }.`}
    >
      <NoticeList notices={notices} />

      <MetricsBand metrics={metrics} />

      <JournalSection
        title='Session journal'
        description='One line per session opened or revoked, with the device and the origin recorded at the time.'
        rows={filtered}
        total={events.length}
        columns={[
          columns.event,
          columns.outcome,
          columns.actor,
          columns.origin,
          columns.when,
        ]}
        card={eventCard(directory)}
        getKey={(e) => e.id}
        loading={isLoading}
        filters={filters}
        filter={filter}
        onFilter={setFilter}
        query={query}
        onQuery={setQuery}
        searchPlaceholder='Search by account, IP, device…'
        aggregates={{
          event_type: `${opened.length} opened`,
          status: `${revoked.length} revoked`,
          actor: `${accounts} account${accounts !== 1 ? 's' : ''}`,
        }}
        emptyLabel='No session event'
        emptyHint={`This journal fills as customers sign in and as sessions are revoked. Nothing was recorded over the ${windowLabel}.`}
      />
    </ActivityPage>
  )
}
