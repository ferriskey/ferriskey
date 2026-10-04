import { useMemo } from 'react'
import { Trans, useTranslation } from 'react-i18next'
import { Lock, Unlock } from 'lucide-react'
import { ActivityChart, IconTile, ListingPage, Pill, Section } from '@/components/kit'
import type {
  CardSpec,
  Column,
  ListingAlert,
  ListingMetric,
  PagedListing,
  PaginationMetadata,
} from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import { Schemas } from '@/api/api.client'
import type { WindowEvents } from '@/pages/ciam/activity/feature/use-window-events'
import {
  actorLabel,
  eventDetailSummary,
  eventLabel,
  eventReason,
  formatRelative,
  formatTimestamp,
} from '../event-catalogue'
import { securityEventFilterFields } from '../event-filter-fields'

import SecurityEvent = Schemas.SecurityEvent
import DailyActivityStats = Schemas.DailyActivityStats
import type { RealmDirectory } from '@/hooks/use-realm-directory'

export interface PageSecurityEventsProps {
  events: SecurityEvent[]
  pagination: PaginationMetadata | undefined
  listing: PagedListing
  recent: WindowEvents
  failures: WindowEvents
  failedInView: number
  activity: DailyActivityStats[]
  isLoading: boolean
  isError: boolean
  directory: RealmDirectory
}

interface RiskyActor {
  identifier: string
  count: number
  ip?: string | null
  lastSeen: string
}

const riskyActors = (events: SecurityEvent[]): RiskyActor[] => {
  const grouped = new Map<string, { count: number; lastSeen: string; ip?: string | null }>()

  events
    .filter((event) => event.status === 'failure')
    .forEach((event) => {
      const key = event.actor_id ?? ''
      const existing = grouped.get(key)
      if (!existing) {
        grouped.set(key, {
          count: 1,
          lastSeen: event.timestamp,
          ip: event.ip_address,
        })
        return
      }
      existing.count += 1
      if (new Date(event.timestamp).getTime() > new Date(existing.lastSeen).getTime()) {
        existing.lastSeen = event.timestamp
        existing.ip = event.ip_address ?? existing.ip
      }
    })

  return Array.from(grouped.entries())
    .map(([identifier, data]) => ({ identifier, ...data }))
    .sort((a, b) => b.count - a.count)
    .slice(0, 3)
}

const latestTimestamp = (events: SecurityEvent[]) => {
  if (events.length === 0) return null
  const latest = events.reduce((acc, event) => {
    if (!acc) return event.timestamp
    return new Date(event.timestamp).getTime() > new Date(acc).getTime()
      ? event.timestamp
      : acc
  }, '')
  if (!latest) return null
  return formatTimestamp(latest)
}

const dominant = (values: (string | null | undefined)[]) => {
  const counts = new Map<string, number>()
  values.forEach((value) => {
    if (!value) return
    counts.set(value, (counts.get(value) ?? 0) + 1)
  })
  const [top] = Array.from(counts.entries()).sort((a, b) => b[1] - a[1])
  return top ?? null
}

const dayKey = (value: string) => {
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return ''
  return `${date.getFullYear()}-${date.getMonth()}-${date.getDate()}`
}

const windowDayKeys = (days: number) => {
  const today = new Date()
  return Array.from({ length: days }, (_, index) => {
    const day = new Date(today)
    day.setDate(today.getDate() - (days - 1 - index))
    return dayKey(day.toISOString())
  })
}

export default function PageSecurityEvents({
  events,
  pagination,
  listing,
  recent,
  failures,
  failedInView,
  activity,
  isLoading,
  isError,
  directory,
}: PageSecurityEventsProps) {
  const { t } = useTranslation('seawatch')
  const windowDays = recent.windowDays

  const columns: Column<SecurityEvent>[] = [
    {
      key: 'event_type',
      header: t('stream.columns.event'),
      render: (e) => (
        <div className='min-w-0'>
          <span>{eventLabel(e)}</span>
          <p className='truncate font-mono-ui text-[11px] text-neutral-400 dark:text-neutral-500'>
            {e.event_type}
          </p>
        </div>
      ),
      sortKey: 'event_type',
    },
    {
      key: 'status',
      header: t('stream.columns.status'),
      render: (e) => {
        const reason = eventReason(e)
        return (
          <div className='min-w-0'>
            <Pill tone={e.status === 'failure' ? 'danger' : 'success'} mono>
              {e.status}
            </Pill>
            {reason && (reason.reason || reason.errorCode) && (
              <p className='mt-0.5 truncate text-xs text-fk-danger'>
                {reason.errorCode && (
                  <span className='font-mono-ui'>{reason.errorCode}</span>
                )}
                {reason.errorCode && reason.reason ? ' — ' : ''}
                {reason.reason}
              </p>
            )}
          </div>
        )
      },
      sortKey: 'status',
    },
    {
      key: 'actor',
      header: t('stream.columns.actor'),
      render: (e) => {
        const identifier = actorLabel(e)
        if (!identifier)
          return (
            <span className='text-xs text-neutral-400 dark:text-neutral-500'>
              {t('stream.actor.unattributed')}
            </span>
          )
        const name = directory.label(e.actor_id, e.actor_type)
        return (
          <div className='min-w-0'>
            <span className={
                name
                  ? 'text-neutral-700 dark:text-neutral-300'
                  : 'font-mono-ui text-xs text-neutral-600 dark:text-neutral-400'
              }>
              {name ?? identifier}
            </span>
            <p className='truncate font-mono-ui text-[11px] text-neutral-400 dark:text-neutral-500'>
              {name ? identifier : (e.actor_type ?? t('stream.actor.unknown_type'))}
            </p>
          </div>
        )
      },
    },
    {
      key: 'target',
      header: t('stream.columns.target'),
      render: (e) => {
        const identifier = e.target_id ?? e.resource
        if (!identifier)
          return (
            <span className='text-xs text-neutral-400 dark:text-neutral-500'>
              {t('stream.target.none')}
            </span>
          )
        const name = directory.label(e.target_id, e.target_type) ?? e.resource
        const resolved = name && name !== identifier
        return (
          <div className='min-w-0'>
            <span className={
                resolved
                  ? 'text-neutral-700 dark:text-neutral-300'
                  : 'font-mono-ui text-xs text-neutral-600 dark:text-neutral-400'
              }>
              {name ?? identifier}
            </span>
            <p className='truncate font-mono-ui text-[11px] text-neutral-400 dark:text-neutral-500'>
              {resolved ? identifier : (e.target_type ?? t('stream.target.unknown_type'))}
            </p>
          </div>
        )
      },
    },
    {
      key: 'ip_address',
      header: t('stream.columns.ip_address'),
      render: (e) =>
        e.ip_address ? (
          <span className='font-mono-ui text-xs text-neutral-600 dark:text-neutral-400'>{e.ip_address}</span>
        ) : (
          <span className='text-xs text-neutral-400 dark:text-neutral-500'>
            {t('stream.ip.not_recorded')}
          </span>
        ),
    },
    {
      key: 'timestamp',
      header: t('stream.columns.timestamp'),
      align: 'right',
      render: (e) => (
        <div className='min-w-0 whitespace-nowrap'>
          <span className='text-neutral-700 dark:text-neutral-300'>{formatRelative(e.timestamp)}</span>
          <p className='tnum truncate font-mono-ui text-[11px] text-neutral-400 dark:text-neutral-500'>
            {formatTimestamp(e.timestamp)}
          </p>
        </div>
      ),
      sortKey: 'timestamp',
    },
  ]

  const card: CardSpec<SecurityEvent> = {
    avatar: (e) => (
      <IconTile tone={e.status === 'failure' ? 'danger' : 'success'}>
        {e.status === 'failure' ? (
          <Lock className='size-4' strokeWidth={1.75} />
        ) : (
          <Unlock className='size-4' strokeWidth={1.75} />
        )}
      </IconTile>
    ),
    title: (e) => eventLabel(e),
    subtitle: (e) => e.event_type,
    badges: (e) => (
      <>
        <Pill tone={e.status === 'failure' ? 'danger' : 'success'} mono>
          {e.status}
        </Pill>
        {e.resource && (
          <Pill tone='neutral' mono>
            {e.resource}
          </Pill>
        )}
      </>
    ),
    flags: (e) => [
      { label: t('stream.card.flags.actor'), on: Boolean(e.actor_id) },
      { label: t('stream.card.flags.target'), on: Boolean(e.target_id ?? e.resource) },
      { label: t('stream.card.flags.origin'), on: Boolean(e.ip_address) },
    ],
    footer: (e) => (
      <>
        <span className='truncate'>
          {eventDetailSummary(e) ?? e.user_agent ?? t('stream.card.no_detail')}
        </span>
        <span className='tnum shrink-0 pl-3 text-right'>
          {formatTimestamp(e.timestamp)}
        </span>
      </>
    ),
  }

  const total = recent.total
  const failed = failures.total
  const successes = Math.max(total - failed, 0)
  const successRate = total ? Math.round((successes / total) * 100) : 0
  const uniqueActors = new Set(recent.events.map((e) => e.actor_id ?? 'unknown')).size

  const topFailure = dominant(failures.events.map((e) => eventLabel(e)))
  const topErrorCode = dominant(failures.events.map((e) => eventReason(e)?.errorCode))
  const topFailingResource = dominant(
    failures.events.map((e) => e.resource ?? (e.target_type === 'client' ? e.target_id : null))
  )

  const buckets = useMemo(() => {
    const keys = windowDayKeys(windowDays)
    return keys.map((key) => recent.events.filter((event) => dayKey(event.timestamp) === key))
  }, [recent.events, windowDays])

  const failureBuckets = useMemo(() => {
    const keys = windowDayKeys(windowDays)
    return keys.map((key) => failures.events.filter((event) => dayKey(event.timestamp) === key))
  }, [failures.events, windowDays])

  const sampled = (window: WindowEvents, series: (number | null)[], value: number) =>
    !window.truncated && series.length > 0 ? series : [value, value]

  const successRatePerDay: (number | null)[] = buckets.map((bucket) => {
    if (bucket.length === 0) return null
    const failedInBucket = bucket.filter((event) => event.status === 'failure').length
    return Math.round(((bucket.length - failedInBucket) / bucket.length) * 100)
  })

  const latest = latestTimestamp(recent.events)

  const metrics: ListingMetric[] = [
    {
      key: 'total',
      label: t('metrics.total.label'),
      value: t('metrics.value', { total }),
      hint: latest
        ? t('metrics.total.hint_latest', { timestamp: latest })
        : t('metrics.total.hint_window', { days: windowDays }),
      series: sampled(
        recent,
        buckets.map((bucket) => bucket.length),
        total
      ),
      tone: 'info',
    },
    {
      key: 'failures',
      label: t('metrics.failures.label'),
      value: t('metrics.value', { total: failed }),
      hint: topErrorCode
        ? topErrorCode[0]
        : topFailure
          ? topFailure[0].toLowerCase()
          : t('metrics.failures.hint_none'),
      series: sampled(
        failures,
        failureBuckets.map((bucket) => bucket.length),
        failed
      ),
      tone: 'brand',
    },
    {
      key: 'rate',
      label: t('metrics.rate.label'),
      value: t('metrics.rate.value', { total: successRate }),
      hint: t('metrics.rate.hint', { total: successes }),
      series: sampled(recent, successRatePerDay, successRate),
      tone: 'success',
    },
    {
      key: 'actors',
      label: t('metrics.actors.label'),
      value: t('metrics.value', { total: uniqueActors }),
      hint: t('metrics.actors.hint', { days: windowDays }),
      series: recent.truncated
        ? undefined
        : buckets.map(
            (bucket) => new Set(bucket.map((event) => event.actor_id ?? 'unknown')).size
          ),
      tone: 'violet',
    },
  ]

  const totalLogins = activity.reduce((n, day) => n + day.logins, 0)
  const totalLoginFailures = activity.reduce((n, day) => n + day.login_failures, 0)
  const hasActivity = activity.length > 1 && totalLogins + totalLoginFailures > 0

  const alerts: ListingAlert[] = [
    ...(isError
      ? [
          {
            tone: 'error' as const,
            title: t('alerts.unavailable.title'),
            detail: t('alerts.unavailable.detail'),
          },
        ]
      : []),
    ...(recent.truncated
      ? [
          {
            tone: 'warn' as const,
            title: t('alerts.capped.title', { limit: recent.windowLimit }),
            detail: t('alerts.capped.detail', {
              limit: recent.windowLimit,
              days: windowDays,
            }),
          },
        ]
      : []),
    ...(failed > 0
      ? [
          {
            tone: 'error' as const,
            title: t('alerts.failures.title', {
              count: failed,
              days: windowDays,
            }),
            detail: [
              topFailure
                ? t('alerts.failures.share', {
                    label: topFailure[0],
                    total: topFailure[1],
                  })
                : null,
              topErrorCode ? t('alerts.failures.error_code', { code: topErrorCode[0] }) : null,
              topFailingResource
                ? t('alerts.failures.resource', { resource: topFailingResource[0] })
                : null,
            ]
              .filter(Boolean)
              .join(' · '),
          },
        ]
      : []),
    ...riskyActors(failures.events).map((actor) => ({
      tone: actor.count > 3 ? ('error' as const) : ('warn' as const),
      title: t('alerts.actor.title', {
        actor: actor.identifier || t('alerts.actor.unknown'),
        count: actor.count,
      }),
      detail: actor.ip
        ? t('alerts.actor.detail_with_ip', {
            ip: actor.ip,
            timestamp: formatTimestamp(actor.lastSeen),
          })
        : t('alerts.actor.detail_without_ip', {
            timestamp: formatTimestamp(actor.lastSeen),
          }),
    })),
  ]

  const insights = hasActivity ? (
    <Section
      title={t('activity.title')}
      description={t('activity.description', { days: windowDays })}
      contained={false}
      action={
        <div className='flex items-center gap-3 text-[11px] text-neutral-500 dark:text-neutral-400'>
          <span className='inline-flex items-center gap-1'>
            <span className='size-1.5 rounded-full bg-fk-success' />
            <Trans
              ns='seawatch'
              i18nKey='activity.logins'
              count={totalLogins}
              components={{ value: <span className='tnum' /> }}
            />
          </span>
          <span className='inline-flex items-center gap-1'>
            <span className='size-1.5 rounded-full bg-fk-danger' />
            <Trans
              ns='seawatch'
              i18nKey='activity.failures'
              count={totalLoginFailures}
              components={{ value: <span className='tnum' /> }}
            />
          </span>
        </div>
      }
    >
      <div className={cn(tokens.surface.panel, 'px-2 py-2')}>
        <ActivityChart data={activity} height={130} />
      </div>
    </Section>
  ) : undefined

  return (
    <ListingPage
      title={t('page.title')}
      description={t('page.description', { days: windowDays })}
      loading={isLoading}
      metrics={metrics}
      alerts={alerts}
      insights={insights}
      paged={{ listing, pagination, filterFields: securityEventFilterFields() }}
      searchScopeHint={t('stream.description')}
      rows={events}
      columns={columns}
      card={card}
      getKey={(e) => e.id}
      aggregates={{
        event_type: t('stream.aggregates.events', { count: pagination?.total ?? 0 }),
        status: t('stream.aggregates.failures', { total: failedInView }),
      }}
      emptyLabel={t('stream.empty.label')}
      emptyHint={t('stream.empty.hint')}
    />
  )
}
