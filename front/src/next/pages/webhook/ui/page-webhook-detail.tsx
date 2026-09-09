import { ArrowLeft, Webhook as WebhookIcon } from 'lucide-react'
import { Button } from '@/components/kit/button'
import SaveBar from '@/components/kit/save-bar'
import { IconTile, PageTabs, Pill, Section, type TabItem } from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import { Schemas } from '@/api/api.client'
import WebhookSettingsTab from './webhook-settings-tab'
import WebhookSubscribersField from './webhook-subscribers-field'
import { WEBHOOK_TRIGGER_COUNT } from '../webhook-trigger-catalogue'
import WebhookDeliveriesTab from './webhook-deliveries-tab'
import type { WebhookHeader } from './webhook-headers-field'

import Webhook = Schemas.Webhook
import WebhookTrigger = Schemas.WebhookTrigger
import { formatDateTime } from '@/next/shared/format-date'

export interface PageWebhookDetailProps {
  webhook?: Webhook
  isLoading: boolean
  tab: string
  tabs: TabItem[]
  name: string
  endpoint: string
  description: string
  headers: WebhookHeader[]
  subscribers: WebhookTrigger[]
  errors: Partial<Record<'name' | 'endpoint' | 'description', string>>
  dirtyCount: number
  onNameChange: (v: string) => void
  onEndpointChange: (v: string) => void
  onDescriptionChange: (v: string) => void
  onHeadersChange: (next: WebhookHeader[]) => void
  onSubscribersChange: (next: WebhookTrigger[]) => void
  onBack: () => void
  onDiscard: () => void
  onSave: () => void
  onDelete: () => void
}

export default function PageWebhookDetail({
  webhook,
  isLoading,
  tab,
  tabs,
  name,
  endpoint,
  description,
  headers,
  subscribers,
  errors,
  dirtyCount,
  onNameChange,
  onEndpointChange,
  onDescriptionChange,
  onHeadersChange,
  onSubscribersChange,
  onBack,
  onDiscard,
  onSave,
  onDelete,
}: PageWebhookDetailProps) {
  const container = cn('mx-auto', tokens.page.maxWidth, tokens.page.padding)

  if (isLoading) {
    return (
      <div className={container}>
        <div className='h-4 w-24 animate-pulse rounded bg-neutral-100 dark:bg-neutral-800' />
        <div className='mt-4 flex items-center gap-3'>
          <div className='size-11 animate-pulse rounded-md bg-neutral-100 dark:bg-neutral-800' />
          <div className='space-y-2'>
            <div className='h-5 w-48 animate-pulse rounded bg-neutral-100 dark:bg-neutral-800' />
            <div className='h-4 w-32 animate-pulse rounded bg-neutral-100 dark:bg-neutral-800' />
          </div>
        </div>
      </div>
    )
  }

  if (!webhook) {
    return (
      <div className={container}>
        <Button variant='ghost' size='sm' className='-ml-2 mb-2 text-neutral-500 dark:text-neutral-400' onClick={onBack}>
          <ArrowLeft className='size-3.5' />
          Webhooks
        </Button>
        <div className={cn(tokens.surface.panel, 'grid place-items-center px-6 py-16')}>
          <p className='text-sm font-medium text-neutral-700 dark:text-neutral-300'>Webhook not found</p>
          <p className='mt-1 max-w-sm text-center text-sm text-neutral-500 dark:text-neutral-400'>
            It may have been deleted, or it belongs to another realm.
          </p>
        </div>
      </div>
    )
  }

  const label = webhook.name || webhook.endpoint

  return (
    <div className={container}>
      <Button variant='ghost' size='sm' className='-ml-2 mb-2 text-neutral-500 dark:text-neutral-400' onClick={onBack}>
        <ArrowLeft className='size-3.5' />
        Webhooks
      </Button>

      <div className='flex flex-wrap items-start justify-between gap-4'>
        <div className='flex items-center gap-3'>
          <IconTile tone={webhook.subscribers.length > 0 ? 'info' : 'amber'} className='size-11'>
            <WebhookIcon className='size-5' strokeWidth={1.75} />
          </IconTile>
          <div className='min-w-0'>
            <h1 className={tokens.header.title}>{label}</h1>
            <div className='mt-1.5 flex flex-wrap items-center gap-2'>
              <Pill tone={webhook.subscribers.length > 0 ? 'info' : 'amber'}>
                {webhook.subscribers.length}/{WEBHOOK_TRIGGER_COUNT} events
              </Pill>
              <Pill mono>{webhook.endpoint}</Pill>
            </div>
          </div>
        </div>

        <dl className='shrink-0 text-right text-xs text-neutral-500 dark:text-neutral-400'>
          <dt className='sr-only'>Last triggered</dt>
          <dd className='tnum'>
            {webhook.triggered_at
              ? `Triggered ${formatDateTime(webhook.triggered_at)}`
              : 'Never triggered'}
          </dd>
          <dt className='sr-only'>Identifier</dt>
          <dd className='font-mono-ui text-[11px] text-neutral-400 dark:text-neutral-500'>{webhook.id}</dd>
        </dl>
      </div>

      <PageTabs tabs={tabs} value={tab} className='mt-5'>
        <div className={tokens.page.blockGap}>
          {tab === 'settings' && (
            <WebhookSettingsTab
              label={label}
              name={name}
              endpoint={endpoint}
              description={description}
              headers={headers}
              errors={errors}
              onNameChange={onNameChange}
              onEndpointChange={onEndpointChange}
              onDescriptionChange={onDescriptionChange}
              onHeadersChange={onHeadersChange}
              onDelete={onDelete}
            />
          )}

          {tab === 'events' && (
            <Section
              title='Events to subscribe'
              description={
                subscribers.length > 0
                  ? `${subscribers.length} trigger${subscribers.length > 1 ? 's' : ''} of ${WEBHOOK_TRIGGER_COUNT} notify this endpoint.`
                  : 'No trigger: this endpoint will never be called.'
              }
              contained={false}
            >
              <WebhookSubscribersField
                value={subscribers}
                onChange={onSubscribersChange}
              />
            </Section>
          )}

          {tab === 'deliveries' && <WebhookDeliveriesTab />}
        </div>
      </PageTabs>

      <SaveBar
        show={dirtyCount > 0}
        title={`${dirtyCount} unsaved change${dirtyCount > 1 ? 's' : ''}`}
        description='Review the webhook before applying the changes.'
        onCancel={onDiscard}
        cancelLabel='Discard'
        actions={[{ label: 'Save changes', onClick: onSave }]}
      />
    </div>
  )
}
