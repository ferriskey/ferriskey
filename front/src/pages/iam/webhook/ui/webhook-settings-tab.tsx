import { useTranslation } from 'react-i18next'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import { FieldRow, Section } from '@/components/kit'
import { DangerZone } from '@/components/kit/danger-zone'
import WebhookHeadersField, { type WebhookHeader } from './webhook-headers-field'
import WebhookSigningSecret from './webhook-signing-secret'
import type { RetryPolicyDraft } from '../feature/page-webhook-detail-feature'
import { Schemas } from '@/api/api.client'

import RetryPolicyOverride = Schemas.RetryPolicyOverride

const RETRY_FIELDS: (keyof RetryPolicyDraft)[] = [
  'max_attempts',
  'base_delay_ms',
  'max_delay_ms',
  'max_total_delay_ms',
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
  const { t } = useTranslation('webhook')

  return (
    <>
      <Section title={t('form.general.title')} description={t('form.general.description')}>
        <FieldRow
          label={t('form.name.label')}
          description={t('form.name.description')}
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
          label={t('form.endpoint.label')}
          description={t('form.endpoint.description')}
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
          label={t('form.description.label')}
          description={t('form.description.description')}
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

      <Section title={t('settings.headers.title')} description={t('settings.headers.description')}>
        <div className='py-4'>
          <WebhookHeadersField headers={headers} onChange={onHeadersChange} />
          <p className='mt-3 text-xs text-neutral-500 dark:text-neutral-400'>
            {t('settings.headers.note')}
          </p>
        </div>
      </Section>

      <Section
        title={t('settings.signature.title')}
        description={t('settings.signature.description')}
      >
        <FieldRow
          label={t('settings.signature.secret.label')}
          description={t('settings.signature.secret.description')}
        >
          <WebhookSigningSecret realm={realm} webhookId={webhookId} />
        </FieldRow>
      </Section>

      <Section title={t('settings.retry.title')} description={t('settings.retry.description')}>
        {RETRY_FIELDS.map((field) => (
          <FieldRow
            key={field}
            label={t(`settings.retry.${field}.label`)}
            description={t(`settings.retry.${field}.description`)}
            htmlFor={`retry-${field}`}
          >
            <Input
              id={`retry-${field}`}
              type='number'
              inputMode='numeric'
              min={0}
              value={retryPolicy[field]}
              placeholder={
                effectiveRetryPolicy?.[field] === null ||
                effectiveRetryPolicy?.[field] === undefined
                  ? t('settings.retry.inherited')
                  : String(effectiveRetryPolicy[field])
              }
              onChange={(e) => onRetryPolicyChange({ [field]: e.target.value })}
              className='max-w-[12rem]'
            />
          </FieldRow>
        ))}
      </Section>

      <DangerZone
        resourceName={name}
        label={t('settings.danger.label')}
        description={t('settings.danger.description')}
        buttonLabel={t('settings.danger.button')}
        confirmTitle={t('settings.danger.confirm_title')}
        confirmDescription={t('settings.danger.confirm_description', { name: label })}
        onConfirm={onDelete}
      />
    </>
  )
}
