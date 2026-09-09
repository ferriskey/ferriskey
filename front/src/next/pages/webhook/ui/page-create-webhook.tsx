import { ArrowLeft } from 'lucide-react'
import { Button } from '@/components/kit/button'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import SaveBar from '@/components/kit/save-bar'
import { FieldRow, Section } from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import { Schemas } from '@/api/api.client'
import WebhookHeadersField, { type WebhookHeader } from './webhook-headers-field'
import WebhookSubscribersField from './webhook-subscribers-field'
import { WEBHOOK_TRIGGER_COUNT } from '../webhook-trigger-catalogue'

import WebhookTrigger = Schemas.WebhookTrigger

export interface PageCreateWebhookProps {
  name: string
  endpoint: string
  description: string
  headers: WebhookHeader[]
  subscribers: WebhookTrigger[]
  errors: Partial<Record<'name' | 'endpoint' | 'description', string>>
  canSubmit: boolean
  onNameChange: (v: string) => void
  onEndpointChange: (v: string) => void
  onDescriptionChange: (v: string) => void
  onHeadersChange: (next: WebhookHeader[]) => void
  onSubscribersChange: (next: WebhookTrigger[]) => void
  onBack: () => void
  onSubmit: () => void
}

export default function PageCreateWebhook({
  name,
  endpoint,
  description,
  headers,
  subscribers,
  errors,
  canSubmit,
  onNameChange,
  onEndpointChange,
  onDescriptionChange,
  onHeadersChange,
  onSubscribersChange,
  onBack,
  onSubmit,
}: PageCreateWebhookProps) {
  return (
    <div className={cn('mx-auto', tokens.page.maxWidth, tokens.page.padding)}>
      <Button variant='ghost' size='sm' className='-ml-2 mb-2 text-neutral-500 dark:text-neutral-400' onClick={onBack}>
        <ArrowLeft className='size-3.5' />
        Webhooks
      </Button>

      <div className={tokens.header.spacing}>
        <h1 className={tokens.header.title}>New webhook</h1>
        <p className='mt-0.5 text-sm text-neutral-500 dark:text-neutral-400'>
          Configure an endpoint to receive real-time event notifications.
        </p>
      </div>

      <div className={tokens.page.blockGap}>
        <Section title='General details' description='Webhook configuration.'>
          <FieldRow
            label='Webhook name'
            description='A descriptive name for this webhook.'
            htmlFor='webhook-name'
          >
            <Input
              id='webhook-name'
              value={name}
              onChange={(e) => onNameChange(e.target.value)}
              className='max-w-sm'
              aria-invalid={Boolean(errors.name)}
            />
            {errors.name && <p className='mt-1.5 text-fk-danger text-xs'>{errors.name}</p>}
          </FieldRow>

          <FieldRow
            label='Endpoint URL'
            description='The HTTPS URL that will receive events.'
            htmlFor='webhook-endpoint'
          >
            <Input
              id='webhook-endpoint'
              value={endpoint}
              onChange={(e) => onEndpointChange(e.target.value)}
              className='max-w-lg'
              aria-invalid={Boolean(errors.endpoint)}
            />
            {errors.endpoint && (
              <p className='mt-1.5 text-xs text-fk-danger'>{errors.endpoint}</p>
            )}
          </FieldRow>

          <FieldRow
            label='Description'
            description='Optional description for this webhook.'
            htmlFor='webhook-description'
          >
            <Textarea
              id='webhook-description'
              value={description}
              onChange={(e) => onDescriptionChange(e.target.value)}
              rows={2}
              className='max-w-lg'
            />
            {errors.description && (
              <p className='mt-1.5 text-xs text-fk-danger'>{errors.description}</p>
            )}
          </FieldRow>
        </Section>

        <Section
          title='HTTP headers'
          description='Headers attached to every outgoing call. Their values are never returned on read.'
        >
          <div className='py-4'>
            <WebhookHeadersField headers={headers} onChange={onHeadersChange} />
          </div>
        </Section>

        <Section
          title='Events to subscribe'
          description={
            subscribers.length > 0
              ? `${subscribers.length} trigger${subscribers.length > 1 ? 's' : ''} of ${WEBHOOK_TRIGGER_COUNT} will notify this endpoint.`
              : 'No trigger selected: this endpoint would never be called.'
          }
          contained={false}
        >
          <WebhookSubscribersField value={subscribers} onChange={onSubscribersChange} />
        </Section>
      </div>

      <SaveBar
        show={canSubmit}
        title='Create webhook'
        description='Create a webhook for this realm.'
        onCancel={onBack}
        actions={[{ label: 'Create', onClick: onSubmit }]}
      />
    </div>
  )
}
