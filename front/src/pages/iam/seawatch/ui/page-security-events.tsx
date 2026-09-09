import { useMemo, useState } from 'react'
import {
  AlertTriangle,
  CheckCircle2,
  LayoutGrid,
  List,
  Lock,
  Search,
  Unlock,
} from 'lucide-react'
import {
  ActivityChart,
  Button,
  DataView,
  IconTile,
  MetricsBand,
  Pill,
  Section,
} from '@/components/kit'
import type {
  CardSpec,
  Column,
  ListingAlert,
  ListingQuery,
  Metric,
  ViewMode,
} from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import { Schemas } from '@/api/api.client'
import {
  actorLabel,
  eventDetailSummary,
  eventLabel,
  eventReason,
  formatRelative,
  formatTimestamp,
} from '../event-catalogue'

import SecurityEvent = Schemas.SecurityEvent
import DailyActivityStats = Schemas.DailyActivityStats
import type { RealmDirectory } from '@/hooks/use-realm-directory'

export interface PageSecurityEventsProps {
  events: SecurityEvent[]
  activity: DailyActivityStats[]
  isLoading: boolean
  isError: boolean
  windowDays: number
  windowLimit: number
  truncated: boolean
  listing: ListingQuery
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
      const key = event.actor_id ?? 'Unknown actor'
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
  if (events.length === 0) return 'No activity yet'
  const latest = events.reduce((acc, event) => {
    if (!acc) return event.timestamp
    return new Date(event.timestamp).getTime() > new Date(acc).getTime()
      ? event.timestamp
      : acc
  }, '')
  if (!latest) return 'No activity yet'
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

const eventFilters: {
  key: string
  label: string
  predicate?: (event: SecurityEvent) => boolean
}[] = [
  { key: 'all', label: 'All' },
  {
    key: 'failures',
    label: 'Failures',
    predicate: (event) => event.status === 'failure',
  },
  { key: 'authentication', label: 'Authentication' },
  { key: 'credentials', label: 'Credentials' },
  { key: 'administration', label: 'Administration' },
]

const searchIn = (event: SecurityEvent) =>
  `${event.event_type} ${event.actor_id ?? ''} ${event.target_id ?? ''} ${event.target_type ?? ''} ${event.resource ?? ''} ${event.ip_address ?? ''} ${event.user_agent ?? ''} ${event.status}`

export default function PageSecurityEvents({
  events,
  activity,
  isLoading,
  isError,
  windowDays,
  windowLimit,
  truncated,
  listing,
  directory,
}: PageSecurityEventsProps) {
  const [view, setView] = useState<ViewMode>('list')
  const [query, setQuery] = useState('')

  const windowLabel = `last ${windowDays} days`

  const columns: Column<SecurityEvent>[] = [
    {
      key: 'event_type',
      header: 'Event',
      render: (e) => (
        <div className='min-w-0'>
          <span>{eventLabel(e)}</span>
          <p className='truncate font-mono-ui text-[11px] text-neutral-400 dark:text-neutral-500'>
            {e.event_type}
          </p>
        </div>
      ),
      sortValue: (e) => eventLabel(e),
    },
    {
      key: 'status',
      header: 'Outcome',
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
      sortValue: (e) => e.status,
    },
    {
      key: 'actor',
      header: 'Actor',
      render: (e) => {
        const identifier = actorLabel(e)
        if (!identifier)
          return <span className='text-xs text-neutral-400 dark:text-neutral-500'>unattributed</span>
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
              {name ? identifier : (e.actor_type ?? 'unknown type')}
            </p>
          </div>
        )
      },
      sortValue: (e) => directory.label(e.actor_id, e.actor_type) ?? actorLabel(e) ?? '',
    },
    {
      key: 'target',
      header: 'Target',
      render: (e) => {
        const identifier = e.target_id ?? e.resource
        if (!identifier) return <span className='text-xs text-neutral-400 dark:text-neutral-500'>none</span>
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
              {resolved ? identifier : (e.target_type ?? 'unknown type')}
            </p>
          </div>
        )
      },
      sortValue: (e) =>
        directory.label(e.target_id, e.target_type) ?? e.target_id ?? e.resource ?? '',
    },
    {
      key: 'ip_address',
      header: 'IP address',
      render: (e) =>
        e.ip_address ? (
          <span className='font-mono-ui text-xs text-neutral-600 dark:text-neutral-400'>{e.ip_address}</span>
        ) : (
          <span className='text-xs text-neutral-400 dark:text-neutral-500'>not recorded</span>
        ),
      sortValue: (e) => e.ip_address ?? '',
    },
    {
      key: 'timestamp',
      header: 'When',
      align: 'right',
      render: (e) => (
        <div className='min-w-0 whitespace-nowrap'>
          <span className='text-neutral-700 dark:text-neutral-300'>{formatRelative(e.timestamp)}</span>
          <p className='tnum truncate font-mono-ui text-[11px] text-neutral-400 dark:text-neutral-500'>
            {formatTimestamp(e.timestamp)}
          </p>
        </div>
      ),
      sortValue: (e) => e.timestamp,
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
      { label: 'Actor identified', on: Boolean(e.actor_id) },
      { label: 'Target recorded', on: Boolean(e.target_id ?? e.resource) },
      { label: 'Origin recorded', on: Boolean(e.ip_address) },
    ],
    footer: (e) => (
      <>
        <span className='truncate'>
          {eventDetailSummary(e) ?? e.user_agent ?? 'no additional detail'}
        </span>
        <span className='tnum shrink-0 pl-3 text-right'>
          {formatTimestamp(e.timestamp)}
        </span>
      </>
    ),
  }

  const failures = events.filter((e) => e.status === 'failure')
  const successes = events.length - failures.length
  const successRate = events.length
    ? Math.round((successes / events.length) * 100)
    : 0
  const uniqueActors = new Set(events.map((e) => e.actor_id ?? 'unknown')).size

  const topFailure = dominant(failures.map((e) => eventLabel(e)))
  const topErrorCode = dominant(failures.map((e) => eventReason(e)?.errorCode))
  const topFailingResource = dominant(
    failures.map((e) => e.resource ?? (e.target_type === 'client' ? e.target_id : null))
  )

  const buckets = useMemo(() => {
    const keys = windowDayKeys(windowDays)
    return keys.map((key) => events.filter((event) => dayKey(event.timestamp) === key))
  }, [events, windowDays])

  const measured = (series: (number | null)[]) =>
    !truncated && series.length > 0 ? series : undefined

  const eventsPerDay = buckets.map((bucket) => bucket.length)
  const failuresPerDay = buckets.map(
    (bucket) => bucket.filter((event) => event.status === 'failure').length
  )
  const actorsPerDay = buckets.map(
    (bucket) => new Set(bucket.map((event) => event.actor_id ?? 'unknown')).size
  )
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
        : events.length > 0
          ? `last ${latestTimestamp(events)}`
          : windowLabel,
      series: measured(eventsPerDay),
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
      series: measured(failuresPerDay),
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
      label: 'Distinct actors',
      value: uniqueActors.toLocaleString(),
      hint: `over the ${windowLabel}`,
      series: measured(actorsPerDay),
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
            title: 'Security events unavailable',
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
              topFailingResource ? `on ${topFailingResource[0]}` : null,
            ]
              .filter(Boolean)
              .join(' · '),
          },
        ]
      : []),
    ...riskyActors(events).map((actor) => ({
      tone: actor.count > 3 ? ('error' as const) : ('warn' as const),
      title: `${actor.identifier} — ${actor.count} failure${actor.count > 1 ? 's' : ''}`,
      detail: actor.ip
        ? `Last seen from ${actor.ip} on ${formatTimestamp(actor.lastSeen)}.`
        : `Last seen on ${formatTimestamp(actor.lastSeen)}, origin not recorded.`,
    })),
  ]

  const filtered = useMemo(() => {
    const predicate = eventFilters.find((f) => f.key === listing.filter)?.predicate
    let out = predicate ? events.filter(predicate) : events
    const needle = query.trim().toLowerCase()
    if (needle) out = out.filter((e) => searchIn(e).toLowerCase().includes(needle))
    return out
  }, [events, listing.filter, query])

  const narrowed = Boolean(query.trim()) || listing.filter !== 'all'
  const filteredOut = narrowed && filtered.length === 0

  const searchBox = (
    <label className='relative flex h-7 w-56 items-center'>
      <Search className='pointer-events-none absolute left-2 size-3.5 text-neutral-400 dark:text-neutral-500' />
      <input
        type='search'
        value={query}
        onChange={(e) => setQuery(e.target.value)}
        placeholder={
          tokens.toolbar.showQuerySyntax
            ? 'event:login_failure  ip:10.0.0.6'
            : 'Filter the stream…'
        }
        className={cn(
          'h-full w-full rounded-md border border-fk-line bg-white dark:bg-fk-surface pl-7 pr-2 outline-none placeholder:text-neutral-400 focus:border-fk-primary-border focus:ring-2 focus:ring-fk-primary/15',
          tokens.toolbar.showQuerySyntax
            ? 'font-mono-ui text-[11px] placeholder:text-neutral-300'
            : 'text-xs'
        )}
      />
    </label>
  )

  return (
    <div className={cn('mx-auto', tokens.page.maxWidth, tokens.page.padding)}>
      <div
        className={cn(
          'flex flex-wrap items-start justify-between gap-3',
          tokens.header.spacing
        )}
      >
        <div className='min-w-0'>
          <h1 className={tokens.header.title}>Sea Watch</h1>
          <p className='mt-0.5 text-sm text-neutral-500 dark:text-neutral-400'>
            {`Security events recorded for this realm over the ${windowLabel}, most recent first.`}
          </p>
        </div>
      </div>

      <div className={tokens.page.sectionGap}>
        {alerts.length > 0 && (
          <ul className='space-y-1'>
            {alerts.map((alert) => (
              <li
                key={alert.title}
                className={cn(
                  'flex items-center gap-2 rounded-sm border px-2.5 py-1.5 text-[13px]',
                  alert.tone === 'error'
                    ? 'border-fk-danger-border bg-fk-danger-soft/40 text-fk-danger'
                    : alert.tone === 'warn'
                      ? 'border-fk-amber-border bg-fk-amber-soft/50 text-fk-amber'
                      : 'border-fk-success-border bg-fk-success-soft/50 text-fk-success'
                )}
              >
                {alert.tone === 'ok' ? (
                  <CheckCircle2 className='size-3.5 shrink-0' strokeWidth={2} />
                ) : (
                  <AlertTriangle className='size-3.5 shrink-0' strokeWidth={2} />
                )}
                <span className='shrink-0 font-medium text-neutral-900 dark:text-neutral-100'>
                  {alert.title}
                </span>
                {alert.detail && (
                  <span className='min-w-0 truncate text-neutral-500 dark:text-neutral-400'>
                    {alert.detail}
                  </span>
                )}
              </li>
            ))}
          </ul>
        )}

        <MetricsBand metrics={metrics} />

        {hasActivity && (
          <Section
            title='Authentication traffic'
            description={`Logins and failures traced by Compass over the ${windowLabel}.`}
            contained={false}
            action={
              <div className='flex items-center gap-3 text-[11px] text-neutral-500 dark:text-neutral-400'>
                <span className='inline-flex items-center gap-1'>
                  <span className='size-1.5 rounded-full bg-fk-success' />
                  <span className='tnum'>{totalLogins}</span> logins
                </span>
                <span className='inline-flex items-center gap-1'>
                  <span className='size-1.5 rounded-full bg-fk-danger' />
                  <span className='tnum'>{totalLoginFailures}</span> failures
                </span>
              </div>
            }
          >
            <div className={cn(tokens.surface.panel, 'px-2 py-2')}>
              <ActivityChart data={activity} height={130} />
            </div>
          </Section>
        )}

        <Section
          title='Event stream'
          description='Event families are applied by the server; the search box narrows the events loaded above.'
          contained={false}
          action={
            <div className='flex flex-wrap items-center justify-end gap-2'>
              <div className='flex gap-1'>
                {eventFilters.map((f) => (
                  <button
                    key={f.key}
                    type='button'
                    onClick={() => listing.setFilter(f.key)}
                    className={cn(
                      'cursor-pointer rounded-md px-2 py-1 text-xs transition-colors',
                      f.key === listing.filter
                        ? 'bg-fk-primary-soft font-medium text-fk-primary-text'
                        : 'text-neutral-500 hover:bg-neutral-100 dark:text-neutral-400 dark:hover:bg-fk-raised'
                    )}
                  >
                    {f.label}
                  </button>
                ))}
              </div>
              {searchBox}
              {tokens.toolbar.showViewToggle && (
                <div className='flex rounded-md border border-fk-line p-0.5'>
                  {(
                    [
                      ['list', List, 'List view'],
                      ['cards', LayoutGrid, 'Card view'],
                    ] as const
                  ).map(([mode, Icon, label]) => (
                    <button
                      key={mode}
                      type='button'
                      onClick={() => setView(mode)}
                      aria-label={label}
                      aria-pressed={view === mode}
                      className={cn(
                        'grid size-6 cursor-pointer place-items-center rounded transition-colors',
                        view === mode
                          ? 'bg-fk-primary-soft text-fk-primary-text'
                          : 'text-neutral-400 hover:text-neutral-700 dark:text-neutral-500 dark:hover:text-neutral-300'
                      )}
                    >
                      <Icon className='size-3.5' />
                    </button>
                  ))}
                </div>
              )}
            </div>
          }
        >
          <div className={tokens.page.sectionGap}>
            <DataView
              rows={filtered}
              columns={columns}
              card={card}
              getKey={(e) => e.id}
              view={view}
              loading={isLoading}
              aggregates={{
                event_type: `${events.length} event${events.length !== 1 ? 's' : ''}`,
                status: `${failures.length} failed`,
                actor: `${uniqueActors} actor${uniqueActors !== 1 ? 's' : ''}`,
              }}
              emptyLabel={filteredOut ? 'No match' : 'No security event'}
              emptyHint={
                filteredOut
                  ? 'No event matches the current search and filter.'
                  : `Sea Watch records every security-relevant action taken in this realm — logins, credential changes, and administrative operations. Nothing was recorded over the ${windowLabel}.`
              }
              emptyAction={
                filteredOut ? (
                  <Button
                    variant='outline'
                    onClick={() => {
                      setQuery('')
                      listing.setFilter('all')
                    }}
                  >
                    Clear filter
                  </Button>
                ) : undefined
              }
            />

            {!isLoading && events.length > 0 && (
              <p className='tnum text-xs text-neutral-400 dark:text-neutral-500'>
                {filtered.length === events.length
                  ? `${events.length} ${events.length > 1 ? 'entries' : 'entry'}`
                  : `${filtered.length} of ${events.length}`}
              </p>
            )}
          </div>
        </Section>
      </div>
    </div>
  )
}
