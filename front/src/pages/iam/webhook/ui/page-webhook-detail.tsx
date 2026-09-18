import type { RetryPolicyDraft } from '../feature/page-webhook-detail-feature'
import { ArrowLeft, Webhook as WebhookIcon } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/kit/button'
import SaveBar from '@/components/kit/save-bar'
import { DetailHeader, IconTile, PageShell, PageTabs, Pill, Section, type TabItem } from '@/components/kit'
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
import { formatDateTime } from '@/utils/format-date'

export interface PageWebhookDetailProps {
  realm: string
  webhook?: Webhook
  isLoading: boolean
  tab: string
  tabs: TabItem[]
  name: string
  endpoint: string
  description: string
  headers: WebhookHeader[]
  retryPolicy: RetryPolicyDraft
  effectiveRetryPolicy?: Schemas.RetryPolicyOverride
  onRetryPolicyChange: (next: Partial<RetryPolicyDraft>) => void
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
  realm,
  webhook,
  isLoading,
  tab,
  tabs,
  name,
  endpoint,
  description,
  headers,
  retryPolicy,
  effectiveRetryPolicy,
  onRetryPolicyChange,
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
  const { t } = useTranslation('webhook')

  if (isLoading) {
    return (
      <PageShell>
        <div className='h-4 w-24 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
        <div className='mt-4 flex items-center gap-3'>
          <div className='size-11 animate-pulse rounded-md bg-neutral-100 dark:bg-fk-raised' />
          <div className='space-y-2'>
            <div className='h-5 w-48 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
            <div className='h-4 w-32 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
          </div>
        </div>
      </PageShell>
    )
  }

  if (!webhook) {
    return (
      <PageShell>
        <Button variant='ghost' size='sm' className='-ml-2 mb-2 text-neutral-500 dark:text-neutral-400' onClick={onBack}>
          <ArrowLeft className='size-3.5' />
          {t('detail.back')}
        </Button>
        <div className={cn(tokens.surface.panel, 'grid place-items-center px-6 py-16')}>
          <p className='text-sm font-medium text-neutral-700 dark:text-neutral-300'>
            {t('detail.not_found.title')}
          </p>
          <p className='mt-1 max-w-sm text-center text-sm text-neutral-500 dark:text-neutral-400'>
            {t('detail.not_found.hint')}
          </p>
        </div>
      </PageShell>
    )
  }

  const label = webhook.name || webhook.endpoint

  return (
    <PageShell>
      <DetailHeader
        onBack={onBack}
        backLabel={t('detail.back')}
        icon={
          <IconTile tone={webhook.subscribers.length > 0 ? 'info' : 'amber'} className='size-11'>
            <WebhookIcon className='size-5' strokeWidth={1.75} />
          </IconTile>
        }
        title={label}
        pills={
          <>
            <Pill tone={webhook.subscribers.length > 0 ? 'info' : 'amber'}>
              {t('events_ratio', {
                selected: webhook.subscribers.length,
                total: WEBHOOK_TRIGGER_COUNT,
              })}
            </Pill>
            <Pill mono>{webhook.endpoint}</Pill>
          </>
        }
        meta={
          <dl className='shrink-0 text-right text-xs text-neutral-500 dark:text-neutral-400'>
            <dt className='sr-only'>{t('detail.meta.last_triggered')}</dt>
            <dd className='tnum'>
              {webhook.triggered_at
                ? t('detail.meta.triggered', { date: formatDateTime(webhook.triggered_at) })
                : t('detail.meta.never')}
            </dd>
            <dt className='sr-only'>{t('detail.meta.identifier')}</dt>
            <dd className='font-mono-ui text-[11px] text-neutral-400 dark:text-neutral-500'>{webhook.id}</dd>
          </dl>
        }
      />

      <PageTabs tabs={tabs} value={tab} className='mt-5'>
        <div className={tokens.page.blockGap}>
          {tab === 'settings' && (
            <WebhookSettingsTab
              realm={realm}
              webhookId={webhook.id}
              retryPolicy={retryPolicy}
              effectiveRetryPolicy={effectiveRetryPolicy}
              onRetryPolicyChange={onRetryPolicyChange}
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
              title={t('detail.events.title')}
              description={
                subscribers.length > 0
                  ? t('detail.events.description', {
                      count: subscribers.length,
                      total: WEBHOOK_TRIGGER_COUNT,
                    })
                  : t('detail.events.description_empty')
              }
              contained={false}
            >
              <WebhookSubscribersField
                value={subscribers}
                onChange={onSubscribersChange}
              />
            </Section>
          )}

          {tab === 'deliveries' && webhook && (
            <WebhookDeliveriesTab realm={realm} webhookId={webhook.id} />
          )}
        </div>
      </PageTabs>

      <SaveBar
        show={dirtyCount > 0}
        title={t('detail.save.title', { count: dirtyCount })}
        description={t('detail.save.description')}
        onCancel={onDiscard}
        cancelLabel={t('detail.save.cancel')}
        actions={[{ label: t('detail.save.action'), onClick: onSave }]}
      />
    </PageShell>
  )
}
