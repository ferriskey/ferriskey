import { useMemo } from 'react'
import { useTranslation } from 'react-i18next'
import { ListingPage } from '@/components/kit'
import type {
  ListingAlert,
  ListingMetric,
  PagedListing,
  PaginationMetadata,
} from '@/components/kit'
import { Schemas } from '@/api/api.client'
import type { RealmDirectory } from '@/hooks/use-realm-directory'
import { eventLabel, eventReason } from '@/pages/iam/seawatch/event-catalogue'
import { securityEventColumnFilters } from '@/pages/iam/seawatch/event-filter-fields'
import { formatTimestamp } from '@/utils/format-date'
import { bucketPerDay, type WindowEvents } from '../feature/use-window-events'
import { eventCard, eventColumns } from './event-journal'

import SecurityEvent = Schemas.SecurityEvent

const CONSOLE_NAMESPACES = ['console', 'seawatch'] as const

export interface PageLogsProps {
  events: SecurityEvent[]
  pagination: PaginationMetadata | undefined
  listing: PagedListing
  recent: WindowEvents
  failures: WindowEvents
  failedInView: number
  isLoading: boolean
  isError: boolean
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

const sampled = (window: WindowEvents, series: (number | null)[], value: number) =>
  !window.truncated && series.length > 0 ? series : [value, value]

export default function PageLogs({
  events,
  pagination,
  listing,
  recent,
  failures,
  failedInView,
  isLoading,
  isError,
  directory,
}: PageLogsProps) {
  const { t } = useTranslation(CONSOLE_NAMESPACES)

  const windowDays = recent.windowDays
  const windowLabel = t('activity.window', { count: windowDays })
  const overWindow = t('activity.over_window', { window: windowLabel })

  const total = recent.total
  const failed = failures.total
  const successes = Math.max(total - failed, 0)
  const successRate = total ? Math.round((successes / total) * 100) : 0
  const uniqueActors = new Set(recent.events.map((e) => e.actor_id ?? 'unknown')).size

  const topFailure = dominant(failures.events.map((e) => eventLabel(e)))
  const topErrorCode = dominant(failures.events.map((e) => eventReason(e)?.errorCode))

  const buckets = useMemo(() => bucketPerDay(recent.events, windowDays), [recent.events, windowDays])
  const failureBuckets = useMemo(
    () => bucketPerDay(failures.events, windowDays),
    [failures.events, windowDays]
  )

  const successRatePerDay: (number | null)[] = buckets.map((bucket) => {
    if (bucket.length === 0) return null
    const failedInBucket = bucket.filter((event) => event.status === 'failure').length
    return Math.round(((bucket.length - failedInBucket) / bucket.length) * 100)
  })

  const latest = latestTimestamp(recent.events)

  const metrics: ListingMetric[] = [
    {
      key: 'total',
      label: t('activity.logs.metrics.events'),
      value: t('number', { value: total }),
      hint: latest ? t('activity.logs.metrics.latest', { timestamp: latest }) : windowLabel,
      series: sampled(
        recent,
        buckets.map((bucket) => bucket.length),
        total
      ),
      tone: 'info',
    },
    {
      key: 'failures',
      label: t('activity.logs.metrics.failures'),
      value: t('number', { value: failed }),
      hint: topErrorCode
        ? topErrorCode[0]
        : topFailure
          ? topFailure[0].toLowerCase()
          : t('activity.no_failure'),
      series: sampled(
        failures,
        failureBuckets.map((bucket) => bucket.length),
        failed
      ),
      tone: 'brand',
    },
    {
      key: 'rate',
      label: t('activity.logs.metrics.rate'),
      value: `${successRate}%`,
      hint: t('activity.logs.metrics.successful', { total: successes }),
      series: sampled(recent, successRatePerDay, successRate),
      tone: 'success',
    },
    {
      key: 'actors',
      label: t('activity.logs.metrics.actors'),
      value: t('number', { value: uniqueActors }),
      hint: overWindow,
      series: recent.truncated
        ? undefined
        : buckets.map(
            (bucket) => new Set(bucket.map((event) => event.actor_id ?? 'unknown')).size
          ),
      tone: 'violet',
    },
  ]

  const alerts: ListingAlert[] = [
    ...(isError
      ? [
          {
            tone: 'error' as const,
            title: t('activity.logs.notices.error.title'),
            detail: t('activity.logs.notices.error.detail'),
          },
        ]
      : []),
    ...(recent.truncated
      ? [
          {
            tone: 'warn' as const,
            title: t('activity.capped.title', { limit: recent.windowLimit }),
            detail: t('activity.capped.detail_below', {
              window: windowLabel,
              limit: recent.windowLimit,
            }),
          },
        ]
      : []),
    ...(failed > 0
      ? [
          {
            tone: 'error' as const,
            title: t('activity.logs.notices.failures.title', {
              count: failed,
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
  const filters = securityEventColumnFilters()
  const feedColumns = [
    { ...columns.event, sortKey: 'event_type', filters: filters.event_type },
    { ...columns.outcome, sortKey: 'status', filters: filters.status },
    { ...columns.actor, filters: filters.actor },
    { ...columns.target, filters: filters.target },
    columns.origin,
    { ...columns.when, sortKey: 'timestamp' },
  ]

  return (
    <ListingPage
      title={t('activity.logs.title')}
      description={t('activity.logs.description', { window: windowLabel })}
      loading={isLoading}
      metrics={metrics}
      alerts={alerts}
      paged={{ listing, pagination, search: { placeholder: t('seawatch:stream.search_placeholder') } }}
      searchScopeHint={t('activity.logs.feed.description')}
      rows={events}
      columns={feedColumns}
      card={eventCard(directory, t)}
      getKey={(e) => e.id}
      aggregates={{
        event_type: t('activity.logs.aggregates.events', { count: pagination?.total ?? 0 }),
        status: t('activity.logs.aggregates.failed', { total: failedInView }),
      }}
      emptyLabel={t('activity.logs.feed.empty_label')}
      emptyHint={t('activity.logs.feed.empty_hint')}
    />
  )
}
