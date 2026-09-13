import { Loader, Monitor, User } from 'lucide-react'
import { ListingPage, IconTile, Pill } from '@/components/kit'
import type { CardSpec, Column, ListingAlert } from '@/components/kit'
import type { ListingQuery } from '@/components/kit'
import { Schemas } from '@/api/api.client'
import {
  failingStep,
  flowStatusTone,
  formatDateTime,
  formatRelative,
  formatTimestamp,
  formatDuration,
  stepLabel,
} from '../flow-format'

import CompassFlow = Schemas.CompassFlow
import FlowStats = Schemas.FlowStats
import type { RealmDirectory } from '@/hooks/use-realm-directory'

export interface PageFlowsProps {
  flows: CompassFlow[]
  stats: FlowStats | null
  isLoading: boolean
  isError: boolean
  listing: ListingQuery
  directory: RealmDirectory
  flowHref: (flow: CompassFlow) => string
}

const SLOW_FLOW_MS = 5000

export default function PageFlows({
  flows,
  stats,
  isLoading,
  isError,
  listing,
  directory,
  flowHref,
}: PageFlowsProps) {
  const columns: Column<CompassFlow>[] = [
    {
      key: 'started_at',
      header: 'Started',
      render: (f) => (
        <div className='min-w-0 whitespace-nowrap'>
          <span className='text-neutral-700 dark:text-neutral-300'>{formatRelative(f.started_at)}</span>
          <p className='tnum truncate font-mono-ui text-[11px] text-neutral-400 dark:text-neutral-500'>
            {formatTimestamp(f.started_at)}
          </p>
        </div>
      ),
      sortValue: (f) => f.started_at,
    },
    {
      key: 'grant_type',
      header: 'Grant type',
      render: (f) => (
        <Pill tone='neutral' mono>
          {f.grant_type}
        </Pill>
      ),
      sortValue: (f) => f.grant_type,
    },
    {
      key: 'status',
      header: 'Outcome',
      render: (f) => {
        const failing = failingStep(f)
        return (
          <div className='min-w-0'>
            <Pill tone={flowStatusTone[f.status]} mono>
              {f.status === 'pending' && (
                <Loader className='size-3 animate-spin' strokeWidth={2.5} />
              )}
              {f.status}
            </Pill>
            {failing && (
              <p className='mt-0.5 truncate text-xs text-fk-danger'>
                {stepLabel(failing)}
                {failing.error_code ? ` · ${failing.error_code}` : ''}
              </p>
            )}
          </div>
        )
      },
      sortValue: (f) => f.status,
    },
    {
      key: 'client_id',
      header: 'Client',
      render: (f) =>
        f.client_id ? (
          <span className='font-mono-ui text-xs text-neutral-600 dark:text-neutral-400'>{f.client_id}</span>
        ) : (
          <span className='text-xs text-neutral-400 dark:text-neutral-500'>unresolved</span>
        ),
      sortValue: (f) => f.client_id ?? '',
    },
    {
      key: 'user_id',
      header: 'User',
      render: (f) => {
        if (!f.user_id)
          return <span className='text-xs text-neutral-400 dark:text-neutral-500'>never identified</span>
        const name = directory.userLabel(f.user_id)
        if (!name)
          return <span className='font-mono-ui text-xs text-neutral-600 dark:text-neutral-400'>{f.user_id}</span>
        return (
          <div className='min-w-0'>
            <span className='text-neutral-700 dark:text-neutral-300'>{name}</span>
            <p className='truncate font-mono-ui text-[11px] text-neutral-400 dark:text-neutral-500'>{f.user_id}</p>
          </div>
        )
      },
      sortValue: (f) => directory.userLabel(f.user_id) ?? f.user_id ?? '',
    },
    {
      key: 'steps',
      header: 'Steps',
      align: 'right',
      render: (f) => <span className='tnum text-neutral-600 dark:text-neutral-400'>{f.steps.length}</span>,
      sortValue: (f) => f.steps.length,
    },
    {
      key: 'duration_ms',
      header: 'Duration',
      align: 'right',
      render: (f) => (
        <span
          className={
            f.duration_ms != null && f.duration_ms > SLOW_FLOW_MS
              ? 'tnum text-fk-amber'
              : 'tnum text-neutral-600 dark:text-neutral-400'
          }
        >
          {formatDuration(f.duration_ms)}
        </span>
      ),
      sortValue: (f) => f.duration_ms ?? -1,
    },
  ]

  const card: CardSpec<CompassFlow> = {
    avatar: (f) => (
      <IconTile tone={f.user_id ? 'info' : 'amber'}>
        {f.user_id ? (
          <User className='size-4' strokeWidth={1.75} />
        ) : (
          <Monitor className='size-4' strokeWidth={1.75} />
        )}
      </IconTile>
    ),
    title: (f) => formatDateTime(f.started_at),
    subtitle: (f) => f.id,
    badges: (f) => (
      <>
        <Pill tone={flowStatusTone[f.status]} mono>
          {f.status}
        </Pill>
        <Pill tone='neutral' mono>
          {f.grant_type}
        </Pill>
      </>
    ),
    flags: (f) => [
      { label: 'User identified', on: Boolean(f.user_id) },
      { label: 'Execution completed', on: Boolean(f.completed_at) },
      { label: 'No failing step', on: failingStep(f) === null },
    ],
    footer: (f) => (
      <>
        <span className='tnum'>{formatDuration(f.duration_ms)}</span>
        <span className='truncate pl-3 text-right'>
          {f.client_id ?? 'unresolved client'}
        </span>
      </>
    ),
  }

  const failed = flows.filter((f) => f.status === 'failure')
  const expired = flows.filter((f) => f.status === 'expired')

  const dominantFailure = failed.reduce<Record<string, number>>((acc, f) => {
    const step = failingStep(f)
    const key = step ? stepLabel(step) : 'No failing step recorded'
    acc[key] = (acc[key] ?? 0) + 1
    return acc
  }, {})
  const [topFailure] = Object.entries(dominantFailure).sort((a, b) => b[1] - a[1])

  const successRate =
    stats && stats.total > 0
      ? `${Math.round((stats.success_count / stats.total) * 100)}% of total`
      : 'no execution recorded'

  const alerts: ListingAlert[] = [
    ...(isError
      ? [
          {
            tone: 'error' as const,
            title: 'Flows unavailable',
            detail: 'We could not fetch the latest flows. Please try again later.',
          },
        ]
      : []),
    ...(failed.length > 0
      ? [
          {
            tone: 'error' as const,
            title: `${failed.length} failed execution${failed.length > 1 ? 's' : ''}`,
            detail: topFailure
              ? `${topFailure[0]} accounts for ${topFailure[1]} of them.`
              : undefined,
          },
        ]
      : []),
    ...(expired.length > 0
      ? [
          {
            tone: 'warn' as const,
            title: `${expired.length} expired execution${expired.length > 1 ? 's' : ''}`,
            detail:
              'Left for an external provider and never came back — no step failed.',
          },
        ]
      : []),
  ]

  return (
    <ListingPage
      title='Compass'
      description='Trace of the authentication executions of this realm.'
      loading={isLoading}
      metrics={[
        {
          key: 'total',
          label: 'Executions',
          value: stats?.total ?? 0,
          hint: 'recorded in this realm',
        },
        {
          key: 'success',
          label: 'Succeeded',
          value: stats?.success_count ?? 0,
          hint: successRate,
        },
        {
          key: 'failure',
          label: 'Failed',
          value: stats?.failure_count ?? 0,
          hint: 'a step failed',
        },
        {
          key: 'duration',
          label: 'Average duration',
          value:
            stats?.avg_duration_ms != null
              ? formatDuration(Math.round(stats.avg_duration_ms))
              : '—',
          hint: 'completed executions',
        },
      ]}
      alerts={alerts}
      server={{ filter: listing.filter, onFilterChange: listing.setFilter }}
      searchScopeHint='Filters are applied by the server; the search box narrows the executions loaded above.'
      filters={[
        { key: 'failed', label: 'Failed' },
        {
          key: 'unfinished',
          label: 'Unfinished',
          predicate: (f) => !f.completed_at,
        },
        { key: 'anonymous', label: 'No user', predicate: (f) => !f.user_id },
      ]}
      searchPlaceholder='Filter by id, client or user…'
      querySyntax='status:failure  grant:password  client:admin-cli'
      searchIn={(f) =>
        `${f.id} ${f.grant_type} ${f.client_id ?? ''} ${f.user_id ?? ''} ${f.ip_address ?? ''} ${f.user_agent ?? ''} ${f.status}`
      }
      rows={flows}
      columns={columns}
      card={card}
      getKey={(f) => f.id}
      getHref={flowHref}
      aggregates={{
        started_at: `${flows.length} execution${flows.length !== 1 ? 's' : ''}`,
        status: `${failed.length + expired.length} unfinished`,
        steps: flows.reduce((n, f) => n + f.steps.length, 0),
      }}
      emptyLabel='No execution recorded'
      emptyHint='Compass records every authentication attempt made against this realm.'
    />
  )
}
