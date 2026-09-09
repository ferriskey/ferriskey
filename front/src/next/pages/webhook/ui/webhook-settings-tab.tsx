import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import { FieldRow, Section } from '@/components/kit'
import { DangerZone } from '@/components/kit/danger-zone'
import WebhookHeadersField, { type WebhookHeader } from './webhook-headers-field'

export interface WebhookSettingsTabProps {
  label: string
  name: string
  endpoint: string
  description: string
  headers: WebhookHeader[]
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
            className='max-w-lg font-mono-ui text-xs'
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
          <p className='mt-3 text-xs text-neutral-500'>
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
          description='Minted once when the webhook is created. The API never returns it, and no rotation endpoint is exposed yet.'
        >
          <span className='inline-flex max-w-sm items-center rounded-md border border-fk-line bg-neutral-50 px-2.5 py-1.5 font-mono-ui text-sm text-neutral-400'>
            ••••••••••••••••
          </span>
        </FieldRow>
      </Section>

      <DangerZone
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
