import { Building2, Plus } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/kit/button'
import { IconTile, ListingPage, Pill, StatusDot } from '@/components/kit'
import type {
  CardSpec,
  Column,
  FilterField,
  PagedListing,
  PaginationMetadata,
} from '@/components/kit'
import { Schemas } from '@/api/api.client'
import { formatRelative } from '@/utils/format-date'

import Organization = Schemas.Organization

const NAME_SEPARATOR = ', '
const TRUNCATION_MARK = '…'

export interface OrganizationCounts {
  total: number
  enabled: number
  disabled: number
  withDomain: number
}

export interface OrganizationPreview {
  total: number
  names: string[]
}

const previewNames = (preview: OrganizationPreview) =>
  preview.names.join(NAME_SEPARATOR) +
  (preview.total > preview.names.length ? TRUNCATION_MARK : '')

export interface PageOrganizationsOverviewProps {
  organizations: Organization[]
  pagination: PaginationMetadata | undefined
  listing: PagedListing
  isLoading: boolean
  counts: OrganizationCounts
  disabled: OrganizationPreview
  organizationHref: (organization: Organization) => string
  onCreate: () => void
  onReviewDisabled: () => void
}

export default function PageOrganizationsOverview({
  organizations,
  pagination,
  listing,
  isLoading,
  counts,
  disabled,
  organizationHref,
  onCreate,
  onReviewDisabled,
}: PageOrganizationsOverviewProps) {
  const { t } = useTranslation('organization')

  const statusLabel = (enabled: boolean) =>
    enabled ? t('organization.status.enabled') : t('organization.status.disabled')

  const columns: Column<Organization>[] = [
    {
      key: 'name',
      header: t('list.columns.name'),
      render: (o) => o.name,
      sortKey: 'name',
    },
    {
      key: 'alias',
      header: t('list.columns.alias'),
      render: (o) => <span className='font-mono-ui text-xs text-neutral-500 dark:text-neutral-400'>{o.alias}</span>,
      sortKey: 'alias',
    },
    {
      key: 'domain',
      header: t('list.columns.domain'),
      render: (o) =>
        o.domain ? (
          <span className='font-mono-ui text-xs text-neutral-600 dark:text-neutral-400'>{o.domain}</span>
        ) : (
          <span className='text-xs text-neutral-400 dark:text-neutral-500'>
            {t('organization.no_domain')}
          </span>
        ),
    },
    {
      key: 'description',
      header: t('list.columns.description'),
      render: (o) =>
        o.description ? (
          <span className='text-neutral-600 dark:text-neutral-400'>{o.description}</span>
        ) : (
          <span className='font-mono-ui text-xs text-neutral-400 dark:text-neutral-500'>
            {t('organization.identifier', { id: o.id })}
          </span>
        ),
    },
    {
      key: 'status',
      header: t('list.columns.status'),
      render: (o) => (
        <span className='inline-flex items-center gap-1.5 text-xs text-neutral-600 dark:text-neutral-400'>
          <StatusDot on={o.enabled} />
          {statusLabel(o.enabled)}
        </span>
      ),
      sortKey: 'enabled',
    },
    {
      key: 'created',
      header: t('list.columns.created'),
      render: (o) => (
        <span className='tnum text-neutral-600 dark:text-neutral-400'>
          {formatRelative(o.created_at)}
        </span>
      ),
      sortKey: 'created_at',
    },
    {
      key: 'updated',
      header: t('list.columns.updated'),
      render: (o) => (
        <span className='tnum text-neutral-600 dark:text-neutral-400'>
          {formatRelative(o.updated_at)}
        </span>
      ),
      sortKey: 'updated_at',
    },
  ]

  const card: CardSpec<Organization> = {
    avatar: () => (
      <IconTile tone='violet'>
        <Building2 className='size-4' strokeWidth={1.75} />
      </IconTile>
    ),
    title: (o) => o.name,
    subtitle: (o) => o.alias,
    badges: (o) => (
      <>
        <Pill tone={o.enabled ? 'success' : 'neutral'}>
          <StatusDot on={o.enabled} />
          {statusLabel(o.enabled)}
        </Pill>
        {o.domain && <Pill mono>{o.domain}</Pill>}
      </>
    ),
    flags: (o) => [
      { label: t('list.card.flags.reachable'), on: o.enabled },
      { label: t('list.card.flags.domain'), on: Boolean(o.domain) },
      { label: t('list.card.flags.redirect'), on: Boolean(o.redirect_url) },
    ],
    footer: (o) => (
      <span className='truncate'>
        {o.description || t('organization.identifier', { id: o.id })}
      </span>
    ),
  }

  const filterFields: FilterField[] = [
    { kind: 'text', key: 'name', label: t('list.filter_fields.name') },
    { kind: 'text', key: 'alias', label: t('list.filter_fields.alias') },
    { kind: 'text', key: 'domain', label: t('list.filter_fields.domain') },
    { kind: 'boolean', key: 'enabled', label: t('list.filter_fields.enabled') },
    { kind: 'boolean', key: 'has_domain', label: t('list.filter_fields.has_domain') },
  ]

  const createButton = (
    <Button onClick={onCreate}>
      <Plus /> {t('list.create')}
    </Button>
  )

  return (
    <ListingPage
      title={t('list.title')}
      description={t('list.description')}
      loading={isLoading}
      actions={createButton}
      metrics={[
        {
          key: 'total',
          label: t('list.metrics.total.label'),
          value: counts.total,
          hint: t('list.metrics.total.hint', { count: counts.total }),
          series: [counts.total, counts.total],
        },
        {
          key: 'enabled',
          label: t('list.metrics.enabled.label'),
          value: counts.enabled,
          hint:
            counts.enabled > 0 && counts.total > 0
              ? t('list.metrics.enabled.hint', {
                  percent: ((counts.enabled / counts.total) * 100).toFixed(0),
                })
              : t('list.metrics.enabled.empty_hint'),
          series: [counts.enabled, counts.enabled],
        },
        {
          key: 'disabled',
          label: t('list.metrics.disabled.label'),
          value: counts.disabled,
          hint: t('list.metrics.disabled.hint'),
          series: [counts.disabled, counts.disabled],
        },
        {
          key: 'domain',
          label: t('list.metrics.domain.label'),
          value: counts.withDomain,
          hint: t('list.metrics.domain.hint'),
          series: [counts.withDomain, counts.withDomain],
        },
      ]}
      alerts={
        disabled.total
          ? [
              {
                tone: 'warn' as const,
                title: t('list.alerts.disabled.title', { count: disabled.total }),
                detail: t('list.alerts.disabled.detail', {
                  count: disabled.total,
                  names: previewNames(disabled),
                }),
                action: t('list.alerts.disabled.action'),
                onAction: onReviewDisabled,
              },
            ]
          : []
      }
      paged={{ listing, pagination, filterFields }}
      rows={organizations}
      columns={columns}
      card={card}
      getKey={(o) => o.id}
      getHref={organizationHref}
      aggregates={{
        name: t('list.count', { count: counts.total }),
        status: t('list.aggregates.status', { total: counts.enabled }),
      }}
      emptyLabel={t('list.empty.label')}
      emptyHint={t('list.empty.hint')}
      emptyAction={createButton}
    />
  )
}
