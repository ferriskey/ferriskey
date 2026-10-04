import { Loader, Monitor, User } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { ListingPage, IconTile, Pill } from '@/components/kit'
import type {
  CardSpec,
  Column,
  FilterField,
  ListingAlert,
  PagedListing,
  PaginationMetadata,
} from '@/components/kit'
import { Schemas } from '@/api/api.client'
import { clientIdRelationSource } from '@/api/client.relation'
import { userRelationSource } from '@/api/user.relation'
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
import FlowStatus = Schemas.FlowStatus
import type { RealmDirectory } from '@/hooks/use-realm-directory'

export interface FlowCounts {
  failed: number
  expired: number
}

export interface PageFlowsProps {
  flows: CompassFlow[]
  pagination: PaginationMetadata | undefined
  listing: PagedListing
  stats: FlowStats | null
  counts: FlowCounts
  recentFailures: CompassFlow[]
  isLoading: boolean
  isError: boolean
  directory: RealmDirectory
  flowHref: (flow: CompassFlow) => string
}

const SLOW_FLOW_MS = 5000

const FLOW_STATUSES: FlowStatus[] = ['pending', 'success', 'failure', 'expired']

export default function PageFlows({
  flows,
  pagination,
  listing,
  stats,
  counts,
  recentFailures,
  isLoading,
  isError,
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
      sortKey: 'started_at',
    },
    {
      key: 'grant_type',
      header: t('list.columns.grant_type'),
      render: (f) => (
        <Pill tone='neutral' mono>
          {f.grant_type}
        </Pill>
      ),
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
      sortKey: 'status',
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
    },
    {
      key: 'steps',
      header: t('list.columns.steps'),
      align: 'right',
      render: (f) => <span className='tnum text-neutral-600 dark:text-neutral-400'>{f.steps.length}</span>,
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
      sortKey: 'duration_ms',
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

  const filterFields: FilterField[] = [
    { kind: 'text', key: 'ip_address', label: t('list.filter_fields.ip_address') },
    {
      kind: 'enum',
      key: 'status',
      label: t('list.filter_fields.status'),
      options: FLOW_STATUSES.map((status) => ({ value: status, label: status })),
    },
    {
      kind: 'text',
      key: 'grant_type',
      label: t('list.filter_fields.grant_type'),
      placeholder: t('list.filter_fields.grant_type_placeholder'),
    },
    {
      kind: 'relation',
      key: 'client_id',
      label: t('list.filter_fields.client_id'),
      relation: clientIdRelationSource,
    },
    {
      kind: 'relation',
      key: 'user_id',
      label: t('list.filter_fields.user_id'),
      relation: userRelationSource,
    },
    { kind: 'boolean', key: 'identified', label: t('list.card.flags.user') },
    { kind: 'boolean', key: 'completed', label: t('list.card.flags.completed') },
  ]

  const dominantFailure = recentFailures.reduce<Record<string, number>>((acc, f) => {
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
    ...(counts.failed > 0
      ? [
          {
            tone: 'error' as const,
            title: t('list.alerts.failed.title', { count: counts.failed }),
            detail: topFailure
              ? t('list.alerts.failed.detail_recent', {
                  step: topFailure[0],
                  total: topFailure[1],
                  sample: recentFailures.length,
                })
              : undefined,
          },
        ]
      : []),
    ...(counts.expired > 0
      ? [
          {
            tone: 'warn' as const,
            title: t('list.alerts.expired.title', { count: counts.expired }),
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
          series: [stats?.total ?? 0, stats?.total ?? 0],
        },
        {
          key: 'success',
          label: t('list.metrics.success.label'),
          value: stats?.success_count ?? 0,
          hint: successRate,
          series: [stats?.success_count ?? 0, stats?.success_count ?? 0],
        },
        {
          key: 'failure',
          label: t('list.metrics.failure.label'),
          value: stats?.failure_count ?? 0,
          hint: t('list.metrics.failure.hint'),
          series: [stats?.failure_count ?? 0, stats?.failure_count ?? 0],
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
      paged={{ listing, pagination, filterFields }}
      rows={flows}
      columns={columns}
      card={card}
      getKey={(f) => f.id}
      getHref={flowHref}
      aggregates={{
        started_at: t('list.aggregates.executions', { count: pagination?.total ?? 0 }),
        status: t('list.aggregates.unfinished', { total: counts.failed + counts.expired }),
      }}
      emptyLabel={t('list.empty.label')}
      emptyHint={t('list.empty.hint')}
    />
  )
}
