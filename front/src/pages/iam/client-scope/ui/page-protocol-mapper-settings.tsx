import { ArrowLeft } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/kit/button'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import SaveBar from '@/components/kit/save-bar'
import { DetailHeader, FieldRow, PageShell, Pill, Section } from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import {
  FALLBACK_MAPPER_ICON,
  type MapperTemplate,
} from '@/pages/iam/client-scope/constants/protocol-mapper-templates'
import { Schemas } from '@/api/api.client'
import { mapperCategory } from '../mapper-categories'
import MapperConfigFields, { type MapperEntityOptions } from './mapper-config-fields'

import ProtocolMapper = Schemas.ProtocolMapper
import { formatDate } from '@/utils/format-date'


export interface PageProtocolMapperSettingsProps {
  mapper?: ProtocolMapper
  isLoading: boolean
  template: MapperTemplate | null
  name: string
  configJson: string
  configValues: Record<string, string>
  entityOptions: MapperEntityOptions
  nameError?: string
  configError?: string
  hasChanges: boolean
  isPending: boolean
  onNameChange: (v: string) => void
  onConfigJsonChange: (v: string) => void
  onConfigChange: (key: string, value: string) => void
  onBack: () => void
  onReset: () => void
  onSubmit: () => void
}

export default function PageProtocolMapperSettings({
  mapper,
  isLoading,
  template,
  name,
  configJson,
  configValues,
  entityOptions,
  nameError,
  configError,
  hasChanges,
  isPending,
  onNameChange,
  onConfigJsonChange,
  onConfigChange,
  onBack,
  onReset,
  onSubmit,
}: PageProtocolMapperSettingsProps) {
  const { t } = useTranslation('client-scope')

  const backButton = (
    <Button variant='ghost' size='sm' className='-ml-2 mb-2 text-neutral-500 dark:text-neutral-400' onClick={onBack}>
      <ArrowLeft className='size-3.5' />
      {t('mapper_settings.back')}
    </Button>
  )

  if (isLoading) {
    return (
      <PageShell>
        <div className='h-4 w-24 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
        <div className='mt-4 space-y-2'>
          <div className='h-5 w-48 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
          <div className='h-4 w-32 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
        </div>
      </PageShell>
    )
  }

  if (!mapper) {
    return (
      <PageShell>
        {backButton}
        <div className={cn(tokens.surface.panel, 'grid place-items-center px-6 py-16')}>
          <p className='text-sm font-medium text-neutral-700 dark:text-neutral-300'>
            {t('mapper_settings.not_found.title')}
          </p>
          <p className='mt-1 max-w-sm text-center text-sm text-neutral-500 dark:text-neutral-400'>
            {t('mapper_settings.not_found.description')}
          </p>
        </div>
      </PageShell>
    )
  }

  const category = mapperCategory(mapper.mapper_type)

  return (
    <PageShell>
      <DetailHeader
        onBack={onBack}
        backLabel={t('mapper_settings.back')}
        icon={<span className='text-3xl leading-none'>{template?.icon ?? FALLBACK_MAPPER_ICON}</span>}
        title={template ? t(template.nameKey) : mapper.name}
        caption={
          template && (
            <p className='mt-0.5 text-sm text-neutral-500 dark:text-neutral-400'>
              {t(template.descriptionKey)}
            </p>
          )
        }
        pills={
          <>
            <Pill tone={category.tone} mono>
              {t(category.labelKey)}
            </Pill>
            <Pill mono>{mapper.mapper_type}</Pill>
          </>
        }
        meta={
          <dl className='shrink-0 text-right text-xs text-neutral-500 dark:text-neutral-400'>
            <dt className='sr-only'>{t('mapper_settings.meta.created_label')}</dt>
            <dd className='tnum'>
              {t('mapper_settings.meta.created', { date: formatDate(mapper.created_at) })}
            </dd>
            <dt className='sr-only'>{t('mapper_settings.meta.identifier_label')}</dt>
            <dd className='font-mono-ui text-[11px] text-neutral-400 dark:text-neutral-500'>{mapper.id}</dd>
          </dl>
        }
      />

      <div className={cn('mt-5', tokens.page.blockGap)}>
        <Section
          title={t('mapper_settings.general.title')}
          description={t('mapper_settings.general.description')}
        >
          <FieldRow
            label={t('mapper_form.name.label')}
            description={t('mapper_form.name.description')}
            htmlFor='mapper-name'
          >
            <Input
              id='mapper-name'
              value={name}
              onChange={(e) => onNameChange(e.target.value)}
              className='max-w-sm'
              aria-invalid={Boolean(nameError)}
            />
            {nameError && <p className='mt-1.5 text-fk-danger text-xs'>{nameError}</p>}
          </FieldRow>

          <FieldRow
            label={t('mapper_form.type.label')}
            description={t('mapper_form.type.description.settings')}
            htmlFor='mapper-type'
          >
            <Input
              id='mapper-type'
              value={mapper.mapper_type}
              disabled
              className='max-w-sm'
            />
          </FieldRow>
        </Section>

        {template && template.fields.length > 0 ? (
          <Section
            title={t('mapper_settings.settings.title')}
            description={t('mapper_settings.settings.description')}
          >
            <MapperConfigFields
              fields={template.fields}
              values={configValues}
              entityOptions={entityOptions}
              onChange={onConfigChange}
            />
          </Section>
        ) : (
          <Section
            title={t('mapper_settings.raw.title')}
            description={t('mapper_settings.raw.description')}
          >
            <FieldRow
              label={t('mapper_form.config.label')}
              description={t('mapper_form.config.description')}
              htmlFor='mapper-config'
            >
              <Textarea
                id='mapper-config'
                value={configJson}
                rows={10}
                onChange={(e) => onConfigJsonChange(e.target.value)}
                className='max-w-lg'
                aria-invalid={Boolean(configError)}
              />
              {configError && <p className='mt-1.5 text-xs text-fk-danger'>{configError}</p>}
            </FieldRow>
          </Section>
        )}
      </div>

      <SaveBar
        show={hasChanges && !nameError && !configError}
        title={t('mapper_settings.save_bar.title')}
        description={t('mapper_settings.save_bar.description')}
        onCancel={onReset}
        cancelLabel={t('mapper_settings.save_bar.cancel')}
        actions={[
          {
            label: isPending
              ? t('mapper_settings.save_bar.submitting')
              : t('mapper_settings.save_bar.submit'),
            onClick: onSubmit,
          },
        ]}
      />
    </PageShell>
  )
}
