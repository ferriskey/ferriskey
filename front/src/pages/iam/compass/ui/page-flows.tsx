import { Loader, Monitor, User } from 'lucide-react'
import { useTranslation } from 'react-i18next'
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

const QUERY_SYNTAX = 'status:failure  grant:password  client:admin-cli'

export default function PageFlows({
  flows,
  stats,
  isLoading,
  isError,
  listing,
  directory,
  flowHref,
}: PageFlowsProps) {
  const { t } = useTranslation('compass')

  const columns: Column<CompassFlow>[] = [
    {
      key: 'started_at',
      header: t('list.columns.started_at'),
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
      header: t('list.columns.grant_type'),
      render: (f) => (
        <Pill tone='neutral' mono>
          {f.grant_type}
        </Pill>
      ),
      sortValue: (f) => f.grant_type,
    },
    {
      key: 'status',
      header: t('list.columns.status'),
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
      header: t('list.columns.client_id'),
      render: (f) =>
        f.client_id ? (
          <span className='font-mono-ui text-xs text-neutral-600 dark:text-neutral-400'>{f.client_id}</span>
        ) : (
          <span className='text-xs text-neutral-400 dark:text-neutral-500'>
            {t('list.client.unresolved')}
          </span>
        ),
      sortValue: (f) => f.client_id ?? '',
    },
    {
      key: 'user_id',
      header: t('list.columns.user_id'),
      render: (f) => {
        if (!f.user_id)
          return (
            <span className='text-xs text-neutral-400 dark:text-neutral-500'>
              {t('list.user.never_identified')}
            </span>
          )
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
      header: t('list.columns.steps'),
      align: 'right',
      render: (f) => <span className='tnum text-neutral-600 dark:text-neutral-400'>{f.steps.length}</span>,
      sortValue: (f) => f.steps.length,
    },
    {
      key: 'duration_ms',
      header: t('list.columns.duration'),
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
      { label: t('list.card.flags.user'), on: Boolean(f.user_id) },
      { label: t('list.card.flags.completed'), on: Boolean(f.completed_at) },
      { label: t('list.card.flags.no_failure'), on: failingStep(f) === null },
    ],
    footer: (f) => (
      <>
        <span className='tnum'>{formatDuration(f.duration_ms)}</span>
        <span className='truncate pl-3 text-right'>
          {f.client_id ?? t('list.card.unresolved_client')}
        </span>
      </>
    ),
  }

  const failed = flows.filter((f) => f.status === 'failure')
  const expired = flows.filter((f) => f.status === 'expired')

  const dominantFailure = failed.reduce<Record<string, number>>((acc, f) => {
    const step = failingStep(f)
    const key = step ? stepLabel(step) : t('list.alerts.failed.no_step')
    acc[key] = (acc[key] ?? 0) + 1
    return acc
  }, {})
  const [topFailure] = Object.entries(dominantFailure).sort((a, b) => b[1] - a[1])

  const successRate =
    stats && stats.total > 0
      ? t('list.metrics.success.hint', {
          rate: Math.round((stats.success_count / stats.total) * 100),
        })
      : t('list.metrics.success.hint_none')

  const alerts: ListingAlert[] = [
    ...(isError
      ? [
          {
            tone: 'error' as const,
            title: t('list.alerts.unavailable.title'),
            detail: t('list.alerts.unavailable.detail'),
          },
        ]
      : []),
    ...(failed.length > 0
      ? [
          {
            tone: 'error' as const,
            title: t('list.alerts.failed.title', { count: failed.length }),
            detail: topFailure
              ? t('list.alerts.failed.detail', {
                  step: topFailure[0],
                  total: topFailure[1],
                })
              : undefined,
          },
        ]
      : []),
    ...(expired.length > 0
      ? [
          {
            tone: 'warn' as const,
            title: t('list.alerts.expired.title', { count: expired.length }),
            detail: t('list.alerts.expired.detail'),
          },
        ]
      : []),
  ]

  return (
    <ListingPage
      title={t('page.title')}
      description={t('page.description')}
      loading={isLoading}
      metrics={[
        {
          key: 'total',
          label: t('list.metrics.total.label'),
          value: stats?.total ?? 0,
          hint: t('list.metrics.total.hint'),
        },
        {
          key: 'success',
          label: t('list.metrics.success.label'),
          value: stats?.success_count ?? 0,
          hint: successRate,
        },
        {
          key: 'failure',
          label: t('list.metrics.failure.label'),
          value: stats?.failure_count ?? 0,
          hint: t('list.metrics.failure.hint'),
        },
        {
          key: 'duration',
          label: t('list.metrics.duration.label'),
          value:
            stats?.avg_duration_ms != null
              ? formatDuration(Math.round(stats.avg_duration_ms))
              : '—',
          hint: t('list.metrics.duration.hint'),
        },
      ]}
      alerts={alerts}
      server={{ filter: listing.filter, onFilterChange: listing.setFilter }}
      searchScopeHint={t('list.search_hint')}
      filters={[
        { key: 'failed', label: t('list.filters.failed') },
        {
          key: 'unfinished',
          label: t('list.filters.unfinished'),
          predicate: (f) => !f.completed_at,
        },
        {
          key: 'anonymous',
          label: t('list.filters.anonymous'),
          predicate: (f) => !f.user_id,
        },
      ]}
      searchPlaceholder={t('list.search_placeholder')}
      querySyntax={QUERY_SYNTAX}
      searchIn={(f) =>
        `${f.id} ${f.grant_type} ${f.client_id ?? ''} ${f.user_id ?? ''} ${f.ip_address ?? ''} ${f.user_agent ?? ''} ${f.status}`
      }
      rows={flows}
      columns={columns}
      card={card}
      getKey={(f) => f.id}
      getHref={flowHref}
      aggregates={{
        started_at: t('list.aggregates.executions', { count: flows.length }),
        status: t('list.aggregates.unfinished', { total: failed.length + expired.length }),
        steps: flows.reduce((n, f) => n + f.steps.length, 0),
      }}
      emptyLabel={t('list.empty.label')}
      emptyHint={t('list.empty.hint')}
    />
  )
}
