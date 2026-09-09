import { Lock, Unlock } from 'lucide-react'
import type { CardSpec, Column } from '@/components/kit'
import { IconTile, Pill } from '@/components/kit'
import { Schemas } from '@/api/api.client'
import type { RealmDirectory } from '@/next/shared/use-realm-directory'
import {
  actorLabel,
  eventDetailSummary,
  eventLabel,
  eventReason,
  formatRelative,
  formatTimestamp,
} from '@/next/pages/iam/seawatch/event-catalogue'

import SecurityEvent = Schemas.SecurityEvent

const muted = 'text-xs text-neutral-400 dark:text-neutral-500'
const mono = 'font-mono-ui text-xs text-neutral-600 dark:text-neutral-400'
const sub = 'truncate font-mono-ui text-[11px] text-neutral-400 dark:text-neutral-500'
const subNum = `tnum ${sub}`

export const searchEvent = (event: SecurityEvent) =>
  `${event.event_type} ${event.actor_id ?? ''} ${event.target_id ?? ''} ${event.target_type ?? ''} ${event.resource ?? ''} ${event.ip_address ?? ''} ${event.user_agent ?? ''} ${event.status}`

export const detailString = (event: SecurityEvent, key: string) => {
  const details = event.details
  if (typeof details !== 'object' || details === null || Array.isArray(details)) return null
  const value = (details as Record<string, unknown>)[key]
  return typeof value === 'string' ? value : null
}

export const eventColumns = (
  directory: RealmDirectory
): Record<string, Column<SecurityEvent>> => ({
  event: {
    key: 'event_type',
    header: 'Event',
    render: (e) => (
      <div className='min-w-0'>
        <span>{eventLabel(e)}</span>
        <p className={sub}>{e.event_type}</p>
      </div>
    ),
    sortValue: (e) => eventLabel(e),
  },
  outcome: {
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
              {reason.errorCode && <span className='font-mono-ui'>{reason.errorCode}</span>}
              {reason.errorCode && reason.reason ? ' — ' : ''}
              {reason.reason}
            </p>
          )}
        </div>
      )
    },
    sortValue: (e) => e.status,
  },
  actor: {
    key: 'actor',
    header: 'Account',
    render: (e) => {
      const identifier = actorLabel(e)
      if (!identifier) return <span className={muted}>unattributed</span>
      const name = directory.label(e.actor_id, e.actor_type)
      return (
        <div className='min-w-0'>
          <span className={name ? 'text-neutral-700 dark:text-neutral-300' : mono}>
            {name ?? identifier}
          </span>
          <p className={sub}>{name ? identifier : (e.actor_type ?? 'unknown type')}</p>
        </div>
      )
    },
    sortValue: (e) => directory.label(e.actor_id, e.actor_type) ?? actorLabel(e) ?? '',
  },
  target: {
    key: 'target',
    header: 'Target',
    render: (e) => {
      const identifier = e.target_id ?? e.resource
      if (!identifier) return <span className={muted}>none</span>
      const name = directory.label(e.target_id, e.target_type) ?? e.resource
      const resolved = Boolean(name) && name !== identifier
      return (
        <div className='min-w-0'>
          <span className={resolved ? 'text-neutral-700 dark:text-neutral-300' : mono}>
            {name ?? identifier}
          </span>
          <p className={sub}>{resolved ? identifier : (e.target_type ?? 'unknown type')}</p>
        </div>
      )
    },
    sortValue: (e) => directory.label(e.target_id, e.target_type) ?? e.target_id ?? e.resource ?? '',
  },
  origin: {
    key: 'ip_address',
    header: 'Origin',
    render: (e) => (
      <div className='min-w-0'>
        {e.ip_address ? (
          <span className={mono}>{e.ip_address}</span>
        ) : (
          <span className={muted}>not recorded</span>
        )}
        {e.user_agent && <p className={sub}>{e.user_agent}</p>}
      </div>
    ),
    sortValue: (e) => e.ip_address ?? '',
  },
  when: {
    key: 'timestamp',
    header: 'When',
    align: 'right',
    render: (e) => (
      <div className='min-w-0 whitespace-nowrap'>
        <span className='text-neutral-700 dark:text-neutral-300'>
          {formatRelative(e.timestamp)}
        </span>
        <p className={subNum}>{formatTimestamp(e.timestamp)}</p>
      </div>
    ),
    sortValue: (e) => e.timestamp,
  },
})

export const eventCard = (directory: RealmDirectory): CardSpec<SecurityEvent> => ({
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
  subtitle: (e) => directory.label(e.actor_id, e.actor_type) ?? actorLabel(e) ?? e.event_type,
  badges: (e) => (
    <>
      <Pill tone={e.status === 'failure' ? 'danger' : 'success'} mono>
        {e.status}
      </Pill>
      {e.ip_address && (
        <Pill tone='neutral' mono>
          {e.ip_address}
        </Pill>
      )}
    </>
  ),
  flags: (e) => [
    { label: 'Account identified', on: Boolean(e.actor_id) },
    { label: 'Origin recorded', on: Boolean(e.ip_address) },
    { label: 'Device recorded', on: Boolean(e.user_agent) },
  ],
  footer: (e) => (
    <>
      <span className='truncate'>
        {eventDetailSummary(e) ?? e.user_agent ?? 'no additional detail'}
      </span>
      <span className='tnum shrink-0 pl-3 text-right'>{formatTimestamp(e.timestamp)}</span>
    </>
  ),
})
