import { Plus, Webhook as WebhookIcon } from 'lucide-react'
import { Button } from '@/components/kit/button'
import { IconTile, ListingPage, Pill } from '@/components/kit'
import type { CardSpec, Column } from '@/components/kit'
import { Schemas } from '@/api/api.client'
import { WEBHOOK_TRIGGER_COUNT } from '../webhook-trigger-catalogue'

import Webhook = Schemas.Webhook
import { formatDateTime } from '@/next/shared/format-date'

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
  const columns: Column<Webhook>[] = [
    {
      key: 'name',
      header: 'Webhook',
      render: (w) =>
        w.name ? (
          w.name
        ) : (
          <span className='font-mono-ui text-xs text-neutral-500'>{w.endpoint}</span>
        ),
      sortValue: (w) => label(w),
    },
    {
      key: 'endpoint',
      header: 'Endpoint',
      render: (w) => (
        <span className='font-mono-ui text-xs text-neutral-500'>{w.endpoint}</span>
      ),
      sortValue: (w) => w.endpoint,
    },
    {
      key: 'subscribers',
      header: 'Events',
      align: 'right',
      render: (w) =>
        w.subscribers.length > 0 ? (
          <span className='tnum text-neutral-600'>{w.subscribers.length}</span>
        ) : (
          <span className='tnum text-fk-danger'>0</span>
        ),
      sortValue: (w) => w.subscribers.length,
    },
    {
      key: 'triggered_at',
      header: 'Last triggered',
      render: (w) =>
        w.triggered_at ? (
          <span className='font-mono-ui text-xs text-neutral-500'>
            {formatDateTime(w.triggered_at)}
          </span>
        ) : (
          <span className='text-xs text-neutral-400'>never</span>
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
          {w.subscribers.length}/{WEBHOOK_TRIGGER_COUNT} events
        </Pill>
        <Pill tone={w.triggered_at ? 'success' : 'neutral'} mono>
          {w.triggered_at ? 'triggered' : 'never triggered'}
        </Pill>
      </>
    ),
    flags: (w) => [
      { label: 'Subscribes to at least one event', on: w.subscribers.length > 0 },
      { label: 'Already triggered', on: Boolean(w.triggered_at) },
      { label: 'Delivers over HTTPS', on: isSecure(w) },
    ],
    footer: (w) => (
      <>
        <span className='tnum'>{w.subscribers.length} events</span>
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
      <Plus /> New webhook
    </Button>
  )

  return (
    <ListingPage
      title='Webhooks'
      description='Endpoints notified of the events of this realm.'
      loading={isLoading}
      actions={createButton}
      metrics={[
        { key: 'total', label: 'Total', value: webhooks.length, hint: 'endpoints' },
        {
          key: 'subscriptions',
          label: 'Subscriptions',
          value: subscriptions,
          hint: `of ${WEBHOOK_TRIGGER_COUNT} triggers`,
        },
        {
          key: 'never',
          label: 'Never triggered',
          value: neverTriggered.length,
          hint: 'no delivery yet',
        },
        {
          key: 'silent',
          label: 'Without event',
          value: silent.length,
          hint: 'will never fire',
        },
      ]}
      alerts={[
        ...silent.map((w) => ({
          tone: 'warn' as const,
          title: `${label(w)} subscribes to nothing`,
          detail: 'It carries no trigger, so it will never be called.',
          action: 'Review',
        })),
        ...insecure.map((w) => ({
          tone: 'error' as const,
          title: `${label(w)} delivers over plain HTTP`,
          detail: `${w.endpoint} — the payload and its signature headers travel unencrypted.`,
          action: 'Review',
        })),
      ]}
      filters={[
        { key: 'triggered', label: 'Already triggered', predicate: (w) => Boolean(w.triggered_at) },
        { key: 'silent', label: 'Without event', predicate: (w) => w.subscribers.length === 0 },
      ]}
      searchPlaceholder='Filter by name or endpoint…'
      querySyntax='endpoint:hooks.*  event:user.created  events:0'
      searchIn={(w) => `${w.name ?? ''} ${w.endpoint}`}
      rows={webhooks}
      columns={columns}
      card={card}
      getKey={(w) => w.id}
      getHref={webhookHref}
      aggregates={{
        name: `${webhooks.length} webhook${webhooks.length !== 1 ? 's' : ''}`,
        subscribers: subscriptions,
      }}
      emptyLabel='No webhook'
      emptyHint='A webhook notifies an external service of the events of this realm.'
      emptyAction={createButton}
    />
  )
}
