import { Lock, Unlock } from 'lucide-react'
import { ListingPage, IconTile, Pill } from '@/components/kit'
import type { CardSpec, Column, ListingAlert } from '@/components/kit'
import { Schemas } from '@/api/api.client'
import {
  actorLabel,
  eventDetailSummary,
  eventLabel,
  eventReason,
  formatEventTimestamp,
  isAdministrationEvent,
  isAuthenticationEvent,
  isCredentialEvent,
} from '../event-catalogue'

import SecurityEvent = Schemas.SecurityEvent

export interface PageSecurityEventsProps {
  events: SecurityEvent[]
  isLoading: boolean
  isError: boolean
  windowDays: number
  windowLimit: number
  truncated: boolean
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
  return formatEventTimestamp(latest)
}

export default function PageSecurityEvents({
  events,
  isLoading,
  isError,
  windowDays,
  windowLimit,
  truncated,
}: PageSecurityEventsProps) {
  const windowLabel = `last ${windowDays} days`
  const columns: Column<SecurityEvent>[] = [
    {
      key: 'event_type',
      header: 'Event',
      render: (e) => (
        <div className='min-w-0'>
          <span>{eventLabel(e)}</span>
          <p className='truncate font-mono-ui text-[11px] text-neutral-400'>
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
        const actor = actorLabel(e)
        if (!actor) return <span className='text-xs text-neutral-400'>unattributed</span>
        return (
          <div className='min-w-0'>
            <span className='text-neutral-700'>{actor}</span>
            {e.actor_type && (
              <p className='truncate font-mono-ui text-[11px] text-neutral-400'>
                {e.actor_type}
              </p>
            )}
          </div>
        )
      },
      sortValue: (e) => actorLabel(e) ?? '',
    },
    {
      key: 'target',
      header: 'Target',
      render: (e) => {
        const target = e.target_id ?? e.resource
        if (!target) return <span className='text-xs text-neutral-400'>none</span>
        return (
          <div className='min-w-0'>
            <span className='font-mono-ui text-xs text-neutral-600'>{target}</span>
            {e.target_type && (
              <p className='truncate font-mono-ui text-[11px] text-neutral-400'>
                {e.target_type}
              </p>
            )}
          </div>
        )
      },
      sortValue: (e) => e.target_id ?? e.resource ?? '',
    },
    {
      key: 'ip_address',
      header: 'IP address',
      render: (e) =>
        e.ip_address ? (
          <span className='font-mono-ui text-xs text-neutral-600'>{e.ip_address}</span>
        ) : (
          <span className='text-xs text-neutral-400'>not recorded</span>
        ),
      sortValue: (e) => e.ip_address ?? '',
    },
    {
      key: 'timestamp',
      header: 'When',
      align: 'right',
      render: (e) => (
        <span className='tnum text-xs text-neutral-500'>
          {formatEventTimestamp(e.timestamp)}
        </span>
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
          {formatEventTimestamp(e.timestamp)}
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

  const dominantFailure = failures.reduce<Record<string, number>>((acc, e) => {
    const key = eventLabel(e)
    acc[key] = (acc[key] ?? 0) + 1
    return acc
  }, {})
  const [topFailure] = Object.entries(dominantFailure).sort((a, b) => b[1] - a[1])

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
            title: `${failures.length} failed event${failures.length > 1 ? 's' : ''}`,
            detail: topFailure
              ? `${topFailure[0]} accounts for ${topFailure[1]} of them.`
              : undefined,
          },
        ]
      : []),
    ...riskyActors(events).map((actor) => ({
      tone: actor.count > 3 ? ('error' as const) : ('warn' as const),
      title: `${actor.identifier} — ${actor.count} failure${actor.count > 1 ? 's' : ''}`,
      detail: actor.ip
        ? `Last seen from ${actor.ip} on ${formatEventTimestamp(actor.lastSeen)}.`
        : `Last seen on ${formatEventTimestamp(actor.lastSeen)}, origin not recorded.`,
    })),
  ]

  return (
    <ListingPage
      title='Sea Watch'
      description={`Security events recorded for this realm over the ${windowLabel}, most recent first.`}
      loading={isLoading}
      metrics={[
        {
          key: 'total',
          label: 'Events',
          value: events.length.toLocaleString(),
          hint: truncated ? `capped, ${windowLabel}` : windowLabel,
        },
        {
          key: 'failures',
          label: 'Failures',
          value: failures.length.toLocaleString(),
          hint: `${successRate}% success rate`,
        },
        {
          key: 'actors',
          label: 'Distinct actors',
          value: uniqueActors.toLocaleString(),
          hint: `over the ${windowLabel}`,
        },
        {
          key: 'latest',
          label: 'Last event',
          value: latestTimestamp(events),
        },
      ]}
      alerts={alerts}
      filters={[
        { key: 'failures', label: 'Failures', predicate: (e) => e.status === 'failure' },
        { key: 'authentication', label: 'Authentication', predicate: isAuthenticationEvent },
        { key: 'credentials', label: 'Credentials', predicate: isCredentialEvent },
        { key: 'administration', label: 'Administration', predicate: isAdministrationEvent },
      ]}
      searchPlaceholder='Filter by event, actor or IP…'
      querySyntax='event:login_failure  actor:admin  ip:10.0.0.6'
      searchIn={(e) =>
        `${e.event_type} ${e.actor_id ?? ''} ${e.target_id ?? ''} ${e.target_type ?? ''} ${e.resource ?? ''} ${e.ip_address ?? ''} ${e.user_agent ?? ''} ${e.status}`
      }
      rows={events}
      columns={columns}
      card={card}
      getKey={(e) => e.id}
      aggregates={{
        event_type: `${events.length} event${events.length !== 1 ? 's' : ''}`,
        status: `${failures.length} failed`,
        actor: `${uniqueActors} actor${uniqueActors !== 1 ? 's' : ''}`,
      }}
      emptyLabel='No security event'
      emptyHint={`Sea Watch records every security-relevant action taken in this realm — logins, credential changes, and administrative operations. Nothing was recorded over the ${windowLabel}.`}
    />
  )
}
