import { Plus, Webhook as WebhookIcon } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/kit/button'
import { IconTile, ListingPage, Pill } from '@/components/kit'
import type {
  CardSpec,
  Column,
  FilterField,
  PagedListing,
  PaginationMetadata,
} from '@/components/kit'
import { Schemas } from '@/api/api.client'
import { WEBHOOK_TRIGGER_COUNT } from '../webhook-trigger-catalogue'
import { describeDeliveryStatus } from '../webhook-delivery-status'

import Webhook = Schemas.Webhook
import { formatDateTime, formatRelative } from '@/utils/format-date'

const DELIVERY_STATUSES = ['pending', 'delivering', 'succeeded', 'failed'] as const
const NAME_SEPARATOR = ', '
const TRUNCATION_MARK = '…'

export interface WebhookCounts {
  total: number
  never: number
  silent: number
}

export interface WebhookPreview {
  total: number
  names: string[]
}

const previewNames = (preview: WebhookPreview) =>
  preview.names.join(NAME_SEPARATOR) +
  (preview.total > preview.names.length ? TRUNCATION_MARK : '')

export interface PageWebhooksOverviewProps {
  webhooks: Webhook[]
  pagination: PaginationMetadata | undefined
  listing: PagedListing
  isLoading: boolean
  counts: WebhookCounts
  silent: WebhookPreview
  insecure: WebhookPreview
  webhookHref: (webhook: Webhook) => string
  onCreate: () => void
}

const label = (webhook: Webhook) => webhook.name || webhook.endpoint

const isSecure = (webhook: Webhook) => webhook.endpoint.startsWith('https://')

export default function PageWebhooksOverview({
  webhooks,
  pagination,
  listing,
  isLoading,
  counts,
  silent,
  insecure,
  webhookHref,
  onCreate,
}: PageWebhooksOverviewProps) {
  const { t } = useTranslation('webhook')

  const columns: Column<Webhook>[] = [
    {
      key: 'name',
      header: t('list.columns.name'),
      render: (w) =>
        w.name ? (
          w.name
        ) : (
          <span className='font-mono-ui text-xs text-neutral-500 dark:text-neutral-400'>{w.endpoint}</span>
        ),
      sortKey: 'name',
    },
    {
      key: 'endpoint',
      header: t('list.columns.endpoint'),
      render: (w) => (
        <span className='font-mono-ui text-xs text-neutral-500 dark:text-neutral-400'>{w.endpoint}</span>
      ),
      sortKey: 'endpoint',
    },
    {
      key: 'subscribers',
      header: t('list.columns.subscribers'),
      align: 'right',
      render: (w) =>
        w.subscribers.length > 0 ? (
          <span className='tnum text-neutral-600 dark:text-neutral-400'>{w.subscribers.length}</span>
        ) : (
          <span className='tnum text-fk-danger'>0</span>
        ),
    },
    {
      key: 'triggered_at',
      header: t('list.columns.triggered_at'),
      render: (w) =>
        w.triggered_at ? (
          <span className='font-mono-ui text-xs text-neutral-500 dark:text-neutral-400'>
            {formatDateTime(w.triggered_at)}
          </span>
        ) : (
          <span className='text-xs text-neutral-400 dark:text-neutral-500'>
            {t('list.never_triggered')}
          </span>
        ),
      sortKey: 'triggered_at',
    },
    {
      key: 'created',
      header: t('list.columns.created'),
      render: (w) => (
        <span className='tnum text-neutral-600 dark:text-neutral-400'>
          {formatRelative(w.created_at)}
        </span>
      ),
      sortKey: 'created_at',
    },
    {
      key: 'updated',
      header: t('list.columns.updated'),
      render: (w) => (
        <span className='tnum text-neutral-600 dark:text-neutral-400'>
          {formatRelative(w.updated_at)}
        </span>
      ),
      sortKey: 'updated_at',
    },
  ]

  const card: CardSpec<Webhook> = {
    avatar: (w) => (
      <IconTile tone={w.subscribers.length > 0 ? 'info' : 'amber'}>
        <WebhookIcon className='size-4' strokeWidth={1.75} />
      </IconTile>
    ),
    title: (w) => label(w),
    subtitle: (w) => w.endpoint,
    badges: (w) => (
      <>
        <Pill tone={w.subscribers.length > 0 ? 'info' : 'amber'}>
          {t('events_ratio', { selected: w.subscribers.length, total: WEBHOOK_TRIGGER_COUNT })}
        </Pill>
        <Pill tone={w.triggered_at ? 'success' : 'neutral'} mono>
          {w.triggered_at ? t('list.card.triggered') : t('list.card.never_triggered')}
        </Pill>
      </>
    ),
    flags: (w) => [
      { label: t('list.card.flags.subscribed'), on: w.subscribers.length > 0 },
      { label: t('list.card.flags.triggered'), on: Boolean(w.triggered_at) },
      { label: t('list.card.flags.secure'), on: isSecure(w) },
    ],
    footer: (w) => (
      <>
        <span className='tnum'>{t('list.card.footer_events', { total: w.subscribers.length })}</span>
        <span className='truncate pl-3 text-right'>
          {w.description || w.endpoint}
        </span>
      </>
    ),
  }

  const filterFields: FilterField[] = [
    { kind: 'text', key: 'name', label: t('list.filter_fields.name') },
    { kind: 'text', key: 'endpoint', label: t('list.filter_fields.endpoint') },
    {
      kind: 'enum',
      key: 'last_delivery_status',
      label: t('list.filter_fields.last_delivery_status'),
      options: DELIVERY_STATUSES.map((status) => {
        const labelKey = describeDeliveryStatus(status).labelKey
        return { value: status, label: labelKey ? t(labelKey) : status }
      }),
    },
    { kind: 'boolean', key: 'triggered', label: t('list.filter_fields.triggered') },
    { kind: 'boolean', key: 'has_subscribers', label: t('list.filter_fields.has_subscribers') },
    { kind: 'boolean', key: 'secure_endpoint', label: t('list.filter_fields.secure_endpoint') },
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
          hint: t('list.metrics.total.hint'),
          series: [counts.total, counts.total],
        },
        {
          key: 'never',
          label: t('list.metrics.never.label'),
          value: counts.never,
          hint: t('list.metrics.never.hint'),
          series: [counts.never, counts.never],
        },
        {
          key: 'silent',
          label: t('list.metrics.silent.label'),
          value: counts.silent,
          hint: t('list.metrics.silent.hint'),
          series: [counts.silent, counts.silent],
        },
      ]}
      alerts={[
        ...(silent.total
          ? [
              {
                tone: 'warn' as const,
                title: t('list.alerts.silent.title', { count: silent.total }),
                detail: t('list.alerts.silent.detail', { names: previewNames(silent) }),
                action: t('list.alerts.silent.action'),
              },
            ]
          : []),
        ...(insecure.total
          ? [
              {
                tone: 'error' as const,
                title: t('list.alerts.insecure.title', { count: insecure.total }),
                detail: t('list.alerts.insecure.detail', { names: previewNames(insecure) }),
                action: t('list.alerts.insecure.action'),
              },
            ]
          : []),
      ]}
      paged={{ listing, pagination, filterFields }}
      rows={webhooks}
      columns={columns}
      card={card}
      getKey={(w) => w.id}
      getHref={webhookHref}
      aggregates={{
        name: t('list.count', { count: counts.total }),
      }}
      emptyLabel={t('list.empty.label')}
      emptyHint={t('list.empty.hint')}
      emptyAction={createButton}
    />
  )
}
