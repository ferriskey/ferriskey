import { ArrowLeft } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/kit/button'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import SaveBar from '@/components/kit/save-bar'
import { FieldRow, PageShell, Pill, Section } from '@/components/kit'
import { tokens } from '@/styles/style-tokens'
import type { MapperTemplate } from '@/pages/iam/client-scope/constants/protocol-mapper-templates'
import MapperConfigFields, { type MapperEntityOptions } from './mapper-config-fields'

const MAPPER_TYPE_PLACEHOLDER = 'oidc-usermodel-property-mapper'
const CONFIG_JSON_PLACEHOLDER = '{"key": "value"}'

export interface PageCreateProtocolMapperProps {
  template: MapperTemplate
  name: string
  mapperType: string
  configJson: string
  configValues: Record<string, string>
  entityOptions: MapperEntityOptions
  nameError?: string
  configError?: string
  canSubmit: boolean
  isPending: boolean
  onNameChange: (v: string) => void
  onMapperTypeChange: (v: string) => void
  onConfigJsonChange: (v: string) => void
  onConfigChange: (key: string, value: string) => void
  onCancel: () => void
  onSubmit: () => void
}

export default function PageCreateProtocolMapper({
  template,
  name,
  mapperType,
  configJson,
  configValues,
  entityOptions,
  nameError,
  configError,
  canSubmit,
  isPending,
  onNameChange,
  onMapperTypeChange,
  onConfigJsonChange,
  onConfigChange,
  onCancel,
  onSubmit,
}: PageCreateProtocolMapperProps) {
  const { t } = useTranslation('client-scope')
  const hasConfig = template.fields.length > 0 || Boolean(template.isCustom)

  return (
    <PageShell>
      <Button variant='ghost' size='sm' className='-ml-2 mb-2 text-neutral-500 dark:text-neutral-400' onClick={onCancel}>
        <ArrowLeft className='size-3.5' />
        {t('mapper_create.back')}
      </Button>

      <div className='flex items-center gap-3 pb-3'>
        <span className='text-3xl leading-none'>{template.icon}</span>
        <div className='min-w-0'>
          <h1 className={tokens.header.title}>{t(template.nameKey)}</h1>
          <p className='mt-0.5 text-sm text-neutral-500 dark:text-neutral-400'>{t(template.descriptionKey)}</p>
          {!template.isCustom && (
            <div className='mt-1.5'>
              <Pill mono>{template.mapper_type}</Pill>
            </div>
          )}
        </div>
      </div>

      <div className={tokens.page.blockGap}>
        <Section
          title={t('mapper_create.identity.title')}
          description={t('mapper_create.identity.description')}
        >
          <FieldRow
            label={t('mapper_form.name.label')}
            description={t('mapper_form.name.description')}
            htmlFor='new-mapper-name'
          >
            <Input
              id='new-mapper-name'
              value={name}
              onChange={(e) => onNameChange(e.target.value)}
              className='max-w-sm'
              aria-invalid={Boolean(nameError)}
            />
            {nameError && <p className='mt-1.5 text-fk-danger text-xs'>{nameError}</p>}
          </FieldRow>

          {template.isCustom && (
            <FieldRow
              label={t('mapper_form.type.label')}
              description={t('mapper_form.type.description.create')}
              htmlFor='new-mapper-type'
            >
              <Input
                id='new-mapper-type'
                value={mapperType}
                placeholder={MAPPER_TYPE_PLACEHOLDER}
                onChange={(e) => onMapperTypeChange(e.target.value)}
                className='max-w-sm'
              />
            </FieldRow>
          )}
        </Section>

        {hasConfig && (
          <Section
            title={t('mapper_create.settings.title')}
            description={t('mapper_create.settings.description')}
          >
            {template.isCustom ? (
              <FieldRow
                label={t('mapper_form.config.label')}
                description={t('mapper_form.config.description')}
                htmlFor='new-mapper-config'
              >
                <Textarea
                  id='new-mapper-config'
                  value={configJson}
                  placeholder={CONFIG_JSON_PLACEHOLDER}
                  rows={10}
                  onChange={(e) => onConfigJsonChange(e.target.value)}
                  className='max-w-lg'
                  aria-invalid={Boolean(configError)}
                />
                {configError && <p className='mt-1.5 text-xs text-fk-danger'>{configError}</p>}
              </FieldRow>
            ) : (
              <MapperConfigFields
                fields={template.fields}
                values={configValues}
                entityOptions={entityOptions}
                onChange={onConfigChange}
              />
            )}
          </Section>
        )}
      </div>

      <SaveBar
        show={canSubmit}
        title={t('mapper_create.save_bar.title')}
        description={t('mapper_create.save_bar.description')}
        onCancel={onCancel}
        actions={[
          {
            label: isPending
              ? t('mapper_create.save_bar.submitting')
              : t('mapper_create.save_bar.submit'),
            onClick: onSubmit,
          },
        ]}
      />
    </PageShell>
  )
}
