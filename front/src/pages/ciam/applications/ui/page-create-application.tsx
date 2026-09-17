import { ArrowLeft } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Link } from 'react-router-dom'
import { Button } from '@/components/kit/button'
import { Input } from '@/components/ui/input'
import SaveBar from '@/components/kit/save-bar'
import { ChipInput, FieldRow, PageShell, Pill, Section } from '@/components/kit'
import { tokens } from '@/styles/style-tokens'
import { applicationFields, applicationTypeMeta, type ApplicationType } from '../application-types'

const CLIENT_CREDENTIALS_GRANT = 'grant_type=client_credentials'

const DEVICE_CODE_GRANT = 'grant_type=urn:ietf:params:oauth:grant-type:device_code'

export interface CreateApplicationErrors {
  name?: string
  clientId?: string
  callbacks?: string
  origins?: string
}

export interface PageCreateApplicationProps {
  type: ApplicationType
  listUrl: string
  pickerUrl: string
  name: string
  clientId: string
  callbacks: string[]
  origins: string[]
  errors: CreateApplicationErrors
  canSubmit: boolean
  onNameChange: (value: string) => void
  onClientIdChange: (value: string) => void
  onCallbacksChange: (next: string[]) => void
  onOriginsChange: (next: string[]) => void
  onCancel: () => void
  onSubmit: () => void
}

export default function PageCreateApplication({
  type,
  listUrl,
  pickerUrl,
  name,
  clientId,
  callbacks,
  origins,
  errors,
  canSubmit,
  onNameChange,
  onClientIdChange,
  onCallbacksChange,
  onOriginsChange,
  onCancel,
  onSubmit,
}: PageCreateApplicationProps) {
  const { t } = useTranslation('console')
  const meta = applicationTypeMeta(type, t)
  const fields = applicationFields(type, t)

  return (
    <PageShell>
      <Button
        variant='ghost'
        size='sm'
        className='-ml-2 mb-2 text-neutral-500 dark:text-neutral-400'
        asChild
      >
        <Link to={listUrl}>
          <ArrowLeft className='size-3.5' />
          {t('applications.create.back')}
        </Link>
      </Button>

      <div className='flex flex-wrap items-center gap-2 pb-3'>
        <h1 className={tokens.header.title}>{t('applications.create.title')}</h1>
        <Pill tone={meta.tone}>{meta.label}</Pill>
        <Link
          to={pickerUrl}
          className='text-xs text-neutral-500 underline-offset-2 hover:text-fk-primary-text hover:underline dark:text-neutral-400'
        >
          {t('applications.create.change_type')}
        </Link>
      </div>

      <div className={tokens.page.blockGap}>
        <Section
          title={t('applications.create.type.title')}
          description={t('applications.create.type.description')}
        >
          <FieldRow
            label={t('applications.create.type.flow_label')}
            description={meta.description}
          >
            <p className='font-mono-ui text-xs text-neutral-700 dark:text-neutral-300'>
              {meta.flow}
            </p>
          </FieldRow>

          <FieldRow
            label={t('applications.create.type.secret_label')}
            description={
              meta.holdsSecret
                ? t('applications.create.type.secret_held')
                : t('applications.create.type.secret_none')
            }
          >
            <Pill tone={meta.holdsSecret ? 'violet' : 'info'}>
              {meta.holdsSecret
                ? t('applications.create.type.secret_generated')
                : t('applications.create.type.secret_absent')}
            </Pill>
          </FieldRow>
        </Section>

        <Section
          title={t('applications.create.identity.title')}
          description={t('applications.create.identity.description')}
        >
          <FieldRow
            label={t('applications.create.identity.name_label')}
            description={t('applications.create.identity.name_description')}
            htmlFor='new-application-name'
          >
            <Input
              id='new-application-name'
              value={name}
              onChange={(e) => onNameChange(e.target.value)}
              placeholder={t('applications.create.identity.name_placeholder')}
              className='max-w-sm'
              aria-invalid={Boolean(errors.name)}
            />
            {errors.name && <p className='mt-1.5 text-xs text-fk-danger'>{errors.name}</p>}
          </FieldRow>

          <FieldRow
            label={t('applications.create.identity.client_id_label')}
            description={t('applications.create.identity.client_id_description')}
            htmlFor='new-application-client-id'
          >
            <Input
              id='new-application-client-id'
              value={clientId}
              onChange={(e) => onClientIdChange(e.target.value)}
              placeholder={t('applications.create.identity.client_id_placeholder')}
              className='max-w-sm font-mono-ui text-xs'
              aria-invalid={Boolean(errors.clientId)}
            />
            {errors.clientId && (
              <p className='mt-1.5 text-xs text-fk-danger'>{errors.clientId}</p>
            )}
          </FieldRow>
        </Section>

        {(fields.showCallbacks || fields.showOrigins) && (
          <Section
            title={t('applications.create.urls.title')}
            description={t('applications.create.urls.description')}
          >
            {fields.showCallbacks && (
              <FieldRow label={fields.callbacksLabel} description={fields.callbacksHint}>
                <ChipInput
                  values={callbacks}
                  onChange={onCallbacksChange}
                  placeholder={fields.callbacksPlaceholder}
                  emptyHint={
                    fields.callbackRequired
                      ? t('applications.fields.empty.callbacks_required')
                      : t('applications.fields.empty.callbacks_optional')
                  }
                />
                {errors.callbacks && (
                  <p className='mt-1.5 text-xs text-fk-danger'>{errors.callbacks}</p>
                )}
              </FieldRow>
            )}

            {fields.showOrigins && (
              <FieldRow label={fields.originsLabel} description={fields.originsHint}>
                <ChipInput
                  values={origins}
                  onChange={onOriginsChange}
                  placeholder={fields.originsPlaceholder}
                  emptyHint={t('applications.fields.empty.origins')}
                />
                {errors.origins && (
                  <p className='mt-1.5 text-xs text-fk-danger'>{errors.origins}</p>
                )}
              </FieldRow>
            )}
          </Section>
        )}

        {type === 'm2m' && (
          <Section
            title={t('applications.create.m2m.title')}
            description={t('applications.create.m2m.description')}
          >
            <FieldRow
              label={t('applications.create.m2m.how_label')}
              description={t('applications.create.m2m.how_description')}
            >
              <p className='font-mono-ui text-xs text-neutral-700 dark:text-neutral-300'>
                {CLIENT_CREDENTIALS_GRANT}
              </p>
            </FieldRow>
          </Section>
        )}

        {type === 'device' && (
          <Section
            title={t('applications.create.device.title')}
            description={t('applications.create.device.description')}
          >
            <FieldRow
              label={t('applications.create.device.how_label')}
              description={t('applications.create.device.how_description')}
            >
              <p className='font-mono-ui text-xs text-neutral-700 dark:text-neutral-300'>
                {DEVICE_CODE_GRANT}
              </p>
            </FieldRow>
          </Section>
        )}
      </div>

      <SaveBar
        show={canSubmit}
        title={t('applications.create.save_bar.title')}
        description={t('applications.create.save_bar.description')}
        onCancel={onCancel}
        cancelLabel={t('applications.create.save_bar.cancel')}
        actions={[{ label: t('applications.create.save_bar.submit'), onClick: onSubmit }]}
      />
    </PageShell>
  )
}
