import { Plus, Webhook as WebhookIcon } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/kit/button'
import { IconTile, ListingPage, Pill } from '@/components/kit'
import type { CardSpec, Column } from '@/components/kit'
import { Schemas } from '@/api/api.client'
import { WEBHOOK_TRIGGER_COUNT } from '../webhook-trigger-catalogue'

import Webhook = Schemas.Webhook
import { formatDateTime } from '@/utils/format-date'

const QUERY_SYNTAX = 'endpoint:hooks.*  event:user.created  events:0'

export interface PageWebhooksOverviewProps {
  webhooks: Webhook[]
  isLoading: boolean
  webhookHref: (webhook: Webhook) => string
  onCreate: () => void
}

const label = (webhook: Webhook) => webhook.name || webhook.endpoint

const isSecure = (webhook: Webhook) => webhook.endpoint.startsWith('https://')

export default function PageWebhooksOverview({
  webhooks,
  isLoading,
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
      sortValue: (w) => label(w),
    },
    {
      key: 'endpoint',
      header: t('list.columns.endpoint'),
      render: (w) => (
        <span className='font-mono-ui text-xs text-neutral-500 dark:text-neutral-400'>{w.endpoint}</span>
      ),
      sortValue: (w) => w.endpoint,
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
      sortValue: (w) => w.subscribers.length,
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
      sortValue: (w) => w.triggered_at ?? '',
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

  const silent = webhooks.filter((w) => w.subscribers.length === 0)
  const insecure = webhooks.filter((w) => !isSecure(w))
  const neverTriggered = webhooks.filter((w) => !w.triggered_at)
  const subscriptions = webhooks.reduce((n, w) => n + w.subscribers.length, 0)

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
          value: webhooks.length,
          hint: t('list.metrics.total.hint'),
        },
        {
          key: 'subscriptions',
          label: t('list.metrics.subscriptions.label'),
          value: subscriptions,
          hint: t('list.metrics.subscriptions.hint', { total: WEBHOOK_TRIGGER_COUNT }),
        },
        {
          key: 'never',
          label: t('list.metrics.never.label'),
          value: neverTriggered.length,
          hint: t('list.metrics.never.hint'),
        },
        {
          key: 'silent',
          label: t('list.metrics.silent.label'),
          value: silent.length,
          hint: t('list.metrics.silent.hint'),
        },
      ]}
      alerts={[
        ...silent.map((w) => ({
          tone: 'warn' as const,
          title: t('list.alerts.silent.title', { name: label(w) }),
          detail: t('list.alerts.silent.detail'),
          action: t('list.alerts.silent.action'),
        })),
        ...insecure.map((w) => ({
          tone: 'error' as const,
          title: t('list.alerts.insecure.title', { name: label(w) }),
          detail: t('list.alerts.insecure.detail', { endpoint: w.endpoint }),
          action: t('list.alerts.insecure.action'),
        })),
      ]}
      filters={[
        {
          key: 'triggered',
          label: t('list.filters.triggered'),
          predicate: (w) => Boolean(w.triggered_at),
        },
        {
          key: 'silent',
          label: t('list.filters.silent'),
          predicate: (w) => w.subscribers.length === 0,
        },
      ]}
      searchPlaceholder={t('list.search_placeholder')}
      querySyntax={QUERY_SYNTAX}
      searchIn={(w) => `${w.name ?? ''} ${w.endpoint}`}
      rows={webhooks}
      columns={columns}
      card={card}
      getKey={(w) => w.id}
      getHref={webhookHref}
      aggregates={{
        name: t('list.count', { count: webhooks.length }),
        subscribers: subscriptions,
      }}
      emptyLabel={t('list.empty.label')}
      emptyHint={t('list.empty.hint')}
      emptyAction={createButton}
    />
  )
}
