import { ArrowLeft } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/kit/button'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import SaveBar from '@/components/kit/save-bar'
import { FieldRow, PageShell, Section } from '@/components/kit'
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
  const { t } = useTranslation('webhook')

  return (
    <PageShell>
      <Button variant='ghost' size='sm' className='-ml-2 mb-2 text-neutral-500 dark:text-neutral-400' onClick={onBack}>
        <ArrowLeft className='size-3.5' />
        {t('create.back')}
      </Button>

      <div className={tokens.header.spacing}>
        <h1 className={tokens.header.title}>{t('create.title')}</h1>
        <p className='mt-0.5 text-sm text-neutral-500 dark:text-neutral-400'>
          {t('create.description')}
        </p>
      </div>

      <div className={tokens.page.blockGap}>
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
              className='max-w-sm'
              aria-invalid={Boolean(errors.name)}
            />
            {errors.name && <p className='mt-1.5 text-fk-danger text-xs'>{errors.name}</p>}
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

        <Section
          title={t('create.headers.title')}
          description={t('create.headers.description')}
        >
          <div className='py-4'>
            <WebhookHeadersField headers={headers} onChange={onHeadersChange} />
          </div>
        </Section>

        <Section
          title={t('create.events.title')}
          description={
            subscribers.length > 0
              ? t('create.events.description', {
                  count: subscribers.length,
                  total: WEBHOOK_TRIGGER_COUNT,
                })
              : t('create.events.description_empty')
          }
          contained={false}
        >
          <WebhookSubscribersField value={subscribers} onChange={onSubscribersChange} />
        </Section>
      </div>

      <SaveBar
        show={canSubmit}
        title={t('create.save.title')}
        description={t('create.save.description')}
        onCancel={onBack}
        actions={[{ label: t('create.save.action'), onClick: onSubmit }]}
      />
    </PageShell>
  )
}
