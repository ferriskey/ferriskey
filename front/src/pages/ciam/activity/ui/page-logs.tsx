import { useMemo } from 'react'
import { useTranslation } from 'react-i18next'
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

const CONSOLE_NAMESPACES = ['console', 'seawatch'] as const

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

const DETAIL_SEPARATOR = ' · '

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
  const { t } = useTranslation(CONSOLE_NAMESPACES)

  const windowLabel = t('activity.window', { count: windowDays })
  const overWindow = t('activity.over_window', { window: windowLabel })

  const filters = useMemo(
    () => [
      { key: 'all', label: t('activity.filter.all') },
      { key: 'failures', label: t('activity.logs.filters.failures') },
      { key: 'authentication', label: t('activity.logs.filters.authentication') },
      { key: 'credentials', label: t('activity.logs.filters.credentials') },
      { key: 'administration', label: t('activity.logs.filters.administration') },
    ],
    [t]
  )

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

  const latest = latestTimestamp(events)

  const metrics: Metric[] = [
    {
      key: 'total',
      label: t('activity.logs.metrics.events'),
      value: t('number', { value: events.length }),
      hint: truncated
        ? t('activity.logs.metrics.capped', { window: windowLabel })
        : latest
          ? t('activity.logs.metrics.latest', { timestamp: latest })
          : windowLabel,
      series: measured(buckets.map((bucket) => bucket.length)),
      tone: 'info',
    },
    {
      key: 'failures',
      label: t('activity.logs.metrics.failures'),
      value: t('number', { value: failures.length }),
      hint: topErrorCode
        ? topErrorCode[0]
        : topFailure
          ? topFailure[0].toLowerCase()
          : t('activity.no_failure'),
      series: measured(
        buckets.map((bucket) => bucket.filter((event) => event.status === 'failure').length)
      ),
      tone: 'brand',
    },
    {
      key: 'rate',
      label: t('activity.logs.metrics.rate'),
      value: `${successRate}%`,
      hint: t('activity.logs.metrics.successful', { total: successes }),
      series: measured(successRatePerDay),
      tone: 'success',
    },
    {
      key: 'actors',
      label: t('activity.logs.metrics.actors'),
      value: t('number', { value: uniqueActors }),
      hint: overWindow,
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
            title: t('activity.logs.notices.error.title'),
            detail: t('activity.logs.notices.error.detail'),
          },
        ]
      : []),
    ...(truncated
      ? [
          {
            tone: 'warn' as const,
            title: t('activity.capped.title', { limit: windowLimit }),
            detail: t('activity.capped.detail_below', {
              window: windowLabel,
              limit: windowLimit,
            }),
          },
        ]
      : []),
    ...(failures.length > 0
      ? [
          {
            tone: 'error' as const,
            title: t('activity.logs.notices.failures.title', {
              count: failures.length,
              window: windowLabel,
            }),
            detail: [
              topFailure
                ? t('activity.logs.notices.failures.top_failure', {
                    label: topFailure[0],
                    count: topFailure[1],
                  })
                : null,
              topErrorCode
                ? t('activity.logs.notices.failures.top_error_code', { code: topErrorCode[0] })
                : null,
            ]
              .filter(Boolean)
              .join(DETAIL_SEPARATOR),
          },
        ]
      : []),
  ]

  const columns = eventColumns(directory, t)

  const filtered = useMemo(() => {
    const out =
      listing.filter === 'failures' ? events.filter((e) => e.status === 'failure') : events
    const needle = listing.draft.trim().toLowerCase()
    if (!needle) return out
    return out.filter((e) => searchEvent(e).toLowerCase().includes(needle))
  }, [events, listing.filter, listing.draft])

  return (
    <ActivityPage
      title={t('activity.logs.title')}
      description={t('activity.logs.description', { window: windowLabel })}
    >
      <NoticeList notices={notices} />

      <MetricsBand metrics={metrics} />

      <JournalSection
        title={t('activity.logs.feed.title')}
        description={t('activity.logs.feed.description')}
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
        card={eventCard(directory, t)}
        getKey={(e) => e.id}
        loading={isLoading}
        filters={filters}
        filter={listing.filter}
        onFilter={listing.setFilter}
        query={listing.draft}
        onQuery={listing.setDraft}
        searchPlaceholder={t('activity.logs.feed.search_placeholder')}
        aggregates={{
          event_type: t('activity.logs.aggregates.events', { count: events.length }),
          status: t('activity.logs.aggregates.failed', { total: failures.length }),
          actor: t('activity.logs.aggregates.accounts', { count: uniqueActors }),
        }}
        emptyLabel={t('activity.logs.feed.empty_label')}
        emptyHint={t('activity.logs.feed.empty_hint', { window: windowLabel })}
      />
    </ActivityPage>
  )
}
