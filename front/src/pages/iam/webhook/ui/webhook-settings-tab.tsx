import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import { FieldRow, Section } from '@/components/kit'
import { DangerZone } from '@/components/kit/danger-zone'
import WebhookHeadersField, { type WebhookHeader } from './webhook-headers-field'
import WebhookSigningSecret from './webhook-signing-secret'
import type { RetryPolicyDraft } from '../feature/page-webhook-detail-feature'
import { Schemas } from '@/api/api.client'

import RetryPolicyOverride = Schemas.RetryPolicyOverride

const RETRY_FIELDS: {
  key: keyof RetryPolicyDraft
  label: string
  description: string
}[] = [
  {
    key: 'max_attempts',
    label: 'Attempts',
    description: 'How many times to try before giving up. Between 1 and 20.',
  },
  {
    key: 'base_delay_ms',
    label: 'First wait (ms)',
    description: 'How long to wait after the first failure. It doubles each time.',
  },
  {
    key: 'max_delay_ms',
    label: 'Longest wait (ms)',
    description: 'The wait never grows past this.',
  },
  {
    key: 'max_total_delay_ms',
    label: 'Give up after (ms)',
    description: 'Total time a delivery may spend retrying.',
  },
]

export interface WebhookSettingsTabProps {
  label: string
  name: string
  endpoint: string
  description: string
  headers: WebhookHeader[]
  realm: string
  webhookId: string
  retryPolicy: RetryPolicyDraft
  effectiveRetryPolicy?: RetryPolicyOverride
  onRetryPolicyChange: (next: Partial<RetryPolicyDraft>) => void
  errors: Partial<Record<'name' | 'endpoint' | 'description', string>>
  onNameChange: (v: string) => void
  onEndpointChange: (v: string) => void
  onDescriptionChange: (v: string) => void
  onHeadersChange: (next: WebhookHeader[]) => void
  onDelete: () => void
}

export default function WebhookSettingsTab({
  label,
  name,
  endpoint,
  description,
  headers,
  realm,
  webhookId,
  retryPolicy,
  effectiveRetryPolicy,
  onRetryPolicyChange,
  errors,
  onNameChange,
  onEndpointChange,
  onDescriptionChange,
  onHeadersChange,
  onDelete,
}: WebhookSettingsTabProps) {
  return (
    <>
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
            placeholder={endpoint}
            className='max-w-sm'
            aria-invalid={Boolean(errors.name)}
          />
          {errors.name && <p className='mt-1.5 text-xs text-fk-danger'>{errors.name}</p>}
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
        description='Headers attached to every outgoing call. The API never returns the stored values, so nothing is prefilled here.'
      >
        <div className='py-4'>
          <WebhookHeadersField headers={headers} onChange={onHeadersChange} />
          <p className='mt-3 text-xs text-neutral-500 dark:text-neutral-400'>
            Leaving this list empty keeps the headers already stored. Adding one replaces
            the whole set.
          </p>
        </div>
      </Section>

      <Section
        title='Signature'
        description='Each delivery is signed so the receiver can prove the call comes from this realm.'
      >
        <FieldRow
          label='Signing secret'
          description='Stored but never returned. Generating a new one shows it once, so the receiver can be given it.'
        >
          <WebhookSigningSecret realm={realm} webhookId={webhookId} />
        </FieldRow>
      </Section>

      <Section
        title='Retry policy'
        description='Leave a field empty to inherit the realm setting. The greyed-out value is what applies today.'
      >
        {RETRY_FIELDS.map((field) => (
          <FieldRow
            key={field.key}
            label={field.label}
            description={field.description}
            htmlFor={`retry-${field.key}`}
          >
            <Input
              id={`retry-${field.key}`}
              type='number'
              inputMode='numeric'
              min={0}
              value={retryPolicy[field.key]}
              placeholder={
                effectiveRetryPolicy?.[field.key] === null ||
                effectiveRetryPolicy?.[field.key] === undefined
                  ? 'Inherited'
                  : String(effectiveRetryPolicy[field.key])
              }
              onChange={(e) => onRetryPolicyChange({ [field.key]: e.target.value })}
              className='max-w-[12rem]'
            />
          </FieldRow>
        ))}
      </Section>

      <DangerZone
        resourceName={name}
        label='Delete this webhook'
        description='Its subscriptions are permanently removed, and no further event is delivered to this endpoint.'
        buttonLabel='Delete webhook'
        confirmTitle='Delete Webhook'
        confirmDescription={`Are you sure you want to delete "${label}"? This action cannot be undone.`}
        onConfirm={onDelete}
      />
    </>
  )
}
