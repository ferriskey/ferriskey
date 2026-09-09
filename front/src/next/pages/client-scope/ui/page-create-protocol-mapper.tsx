import { ArrowLeft } from 'lucide-react'
import { Button } from '@/components/kit/button'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import FloatingActionBar from '@/components/ui/floating-action-bar'
import { FieldRow, Pill, Section } from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import type { MapperTemplate } from '@/pages/client-scope/constants/protocol-mapper-templates'
import MapperConfigFields, { type MapperEntityOptions } from './mapper-config-fields'

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
  const hasConfig = template.fields.length > 0 || Boolean(template.isCustom)

  return (
    <div className={cn('mx-auto', tokens.page.maxWidth, tokens.page.padding)}>
      <Button variant='ghost' size='sm' className='-ml-2 mb-2 text-neutral-500' onClick={onCancel}>
        <ArrowLeft className='size-3.5' />
        Protocol Mappers
      </Button>

      <div className='flex items-center gap-3 pb-3'>
        <span className='text-3xl leading-none'>{template.icon}</span>
        <div className='min-w-0'>
          <h1 className={tokens.header.title}>{template.name}</h1>
          <p className='mt-0.5 text-sm text-neutral-500'>{template.description}</p>
          {!template.isCustom && (
            <div className='mt-1.5'>
              <Pill mono>{template.mapper_type}</Pill>
            </div>
          )}
        </div>
      </div>

      <div className={tokens.page.blockGap}>
        <Section title='Identity' description='How this mapper is named on the scope.'>
          <FieldRow
            label='Name'
            description='Administrative name of the mapper — it is not the claim it writes.'
            htmlFor='new-mapper-name'
          >
            <Input
              id='new-mapper-name'
              value={name}
              onChange={(e) => onNameChange(e.target.value)}
              className='max-w-sm'
              aria-invalid={Boolean(nameError)}
            />
            {nameError && <p className='mt-1.5 text-xs text-fk-danger'>{nameError}</p>}
          </FieldRow>

          {template.isCustom && (
            <FieldRow
              label='Mapper type'
              description='Identifier of the mapper implementation the server runs at token time.'
              htmlFor='new-mapper-type'
            >
              <Input
                id='new-mapper-type'
                value={mapperType}
                placeholder='oidc-usermodel-property-mapper'
                onChange={(e) => onMapperTypeChange(e.target.value)}
                className='max-w-sm font-mono-ui'
              />
            </FieldRow>
          )}
        </Section>

        {hasConfig && (
          <Section
            title='Settings'
            description='Configuration handed to the mapper when a token is issued.'
          >
            {template.isCustom ? (
              <FieldRow
                label='Config'
                description='Raw JSON object, exactly as the mapper expects it.'
                htmlFor='new-mapper-config'
              >
                <Textarea
                  id='new-mapper-config'
                  value={configJson}
                  placeholder='{"key": "value"}'
                  rows={10}
                  onChange={(e) => onConfigJsonChange(e.target.value)}
                  className='max-w-lg font-mono-ui text-xs'
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

      <FloatingActionBar
        show={canSubmit}
        title='Create protocol mapper'
        description='The mapper is added to this client scope right away.'
        onCancel={onCancel}
        actions={[{ label: isPending ? 'Creating…' : 'Create mapper', onClick: onSubmit }]}
      />
    </div>
  )
}
