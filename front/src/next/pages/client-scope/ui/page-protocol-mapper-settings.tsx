import { ArrowLeft } from 'lucide-react'
import { Button } from '@/components/kit/button'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import SaveBar from '@/components/kit/save-bar'
import { FieldRow, Pill, Section } from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import type { MapperTemplate } from '@/pages/client-scope/constants/protocol-mapper-templates'
import { Schemas } from '@/api/api.client'
import { mapperCategory } from '../mapper-categories'
import MapperConfigFields, { type MapperEntityOptions } from './mapper-config-fields'

import ProtocolMapper = Schemas.ProtocolMapper

const formatDate = (iso: string) => new Date(iso).toLocaleDateString('en-GB')

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
  const container = cn('mx-auto', tokens.page.maxWidth, tokens.page.padding)

  const backButton = (
    <Button variant='ghost' size='sm' className='-ml-2 mb-2 text-neutral-500' onClick={onBack}>
      <ArrowLeft className='size-3.5' />
      Protocol Mappers
    </Button>
  )

  if (isLoading) {
    return (
      <div className={container}>
        <div className='h-4 w-24 animate-pulse rounded bg-neutral-100' />
        <div className='mt-4 space-y-2'>
          <div className='h-5 w-48 animate-pulse rounded bg-neutral-100' />
          <div className='h-4 w-32 animate-pulse rounded bg-neutral-100' />
        </div>
      </div>
    )
  }

  if (!mapper) {
    return (
      <div className={container}>
        {backButton}
        <div className={cn(tokens.surface.panel, 'grid place-items-center px-6 py-16')}>
          <p className='text-sm font-medium text-neutral-700'>Protocol mapper not found</p>
          <p className='mt-1 max-w-sm text-center text-sm text-neutral-500'>
            It may have been deleted, or it belongs to another client scope.
          </p>
        </div>
      </div>
    )
  }

  const category = mapperCategory(mapper.mapper_type)

  return (
    <div className={container}>
      {backButton}

      <div className='flex flex-wrap items-start justify-between gap-4'>
        <div className='flex items-center gap-3'>
          <span className='text-3xl leading-none'>{template?.icon ?? '⚙️'}</span>
          <div className='min-w-0'>
            <h1 className={tokens.header.title}>{template?.name ?? mapper.name}</h1>
            {template?.description && (
              <p className='mt-0.5 text-sm text-neutral-500'>{template.description}</p>
            )}
            <div className='mt-1.5 flex flex-wrap items-center gap-2'>
              <Pill tone={category.tone} mono>
                {category.label}
              </Pill>
              <Pill mono>{mapper.mapper_type}</Pill>
            </div>
          </div>
        </div>

        <dl className='shrink-0 text-right text-xs text-neutral-500'>
          <dt className='sr-only'>Created at</dt>
          <dd className='tnum'>Created {formatDate(mapper.created_at)}</dd>
          <dt className='sr-only'>Identifier</dt>
          <dd className='font-mono-ui text-[11px] text-neutral-400'>{mapper.id}</dd>
        </dl>
      </div>

      <div className={cn('mt-5', tokens.page.blockGap)}>
        <Section title='General' description='How this mapper is named on the scope.'>
          <FieldRow
            label='Name'
            description='Administrative name of the mapper — it is not the claim it writes.'
            htmlFor='mapper-name'
          >
            <Input
              id='mapper-name'
              value={name}
              onChange={(e) => onNameChange(e.target.value)}
              className='max-w-sm'
              aria-invalid={Boolean(nameError)}
            />
            {nameError && <p className='mt-1.5 text-fk-danger'>{nameError}</p>}
          </FieldRow>

          <FieldRow
            label='Mapper type'
            description='Set at creation: changing the implementation would invalidate the configuration below.'
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
            title='Settings'
            description='Configuration handed to the mapper when a token is issued.'
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
            title='Configuration (JSON)'
            description='No template matches this mapper type, so its configuration is edited raw.'
          >
            <FieldRow
              label='Config'
              description='Raw JSON object, exactly as the mapper expects it.'
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
        title='Unsaved changes'
        description='Review the mapper before applying the changes.'
        onCancel={onReset}
        cancelLabel='Discard'
        actions={[{ label: isPending ? 'Saving…' : 'Save changes', onClick: onSubmit }]}
      />
    </div>
  )
}
