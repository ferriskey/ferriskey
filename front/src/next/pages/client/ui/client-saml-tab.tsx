import { Trash2 } from 'lucide-react'
import { Button } from '@/components/kit/button'
import { Input } from '@/components/ui/input'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import FloatingActionBar from '@/components/ui/floating-action-bar'
import { ConfirmDeleteAlert } from '@/components/confirm-delete-alert'
import { useConfirmDeleteAlert } from '@/hooks/use-confirm-delete-alert'
import { FieldRow, Section, SwitchField } from '@/components/kit'
import {
  ATTRIBUTE_NAME_FORMAT_OPTIONS,
  BUILT_IN_SOURCE_OPTIONS,
  CUSTOM_ATTRIBUTE_SOURCE,
  NAME_ID_FORMAT_OPTIONS,
  describeAttributeNameFormat,
  describeAttributeSource,
} from '@/lib/saml'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import { Schemas } from '@/api/api.client'

import SamlAttributeMapper = Schemas.SamlAttributeMapper

export interface SamlDraft {
  spEntityId: string
  acsUrl: string
  nameIdFormat: string
  signAssertions: boolean
  signDocuments: boolean
  wantAuthnRequestsSigned: boolean
}

export interface SamlMapperDraft {
  name: string
  source: string
  customKey: string
  nameFormat: string
}

export interface ClientSamlTabProps {
  draft: SamlDraft
  errors: { spEntityId?: string; acsUrl?: string; nameIdFormat?: string }
  mapperDraft: SamlMapperDraft
  mapperErrors: { name?: string; customKey?: string }
  mappers: SamlAttributeMapper[]
  isConfigured: boolean
  isLoading: boolean
  isSaving: boolean
  isCreatingMapper: boolean
  isDeletingMapper: boolean
  dirtyCount: number
  canAddMapper: boolean
  onDraftChange: (patch: Partial<SamlDraft>) => void
  onMapperDraftChange: (patch: Partial<SamlMapperDraft>) => void
  onDiscard: () => void
  onSubmit: () => void
  onAddMapper: () => void
  onAddCommonProfile: () => void
  onDeleteMapper: (mapperId: string) => void
}

const SIGNATURE_FIELDS = [
  {
    key: 'signAssertions',
    label: 'Sign assertions',
    description: 'Sign the assertion itself. Almost every application expects this.',
  },
  {
    key: 'signDocuments',
    label: 'Sign documents',
    description: 'Sign the whole SAML response in addition to the assertion.',
  },
  {
    key: 'wantAuthnRequestsSigned',
    label: 'Require signed authentication requests',
    description:
      'Reject sign-in requests from this application unless they carry a valid signature.',
  },
] as const

export default function ClientSamlTab({
  draft,
  errors,
  mapperDraft,
  mapperErrors,
  mappers,
  isConfigured,
  isLoading,
  isSaving,
  isCreatingMapper,
  isDeletingMapper,
  dirtyCount,
  canAddMapper,
  onDraftChange,
  onMapperDraftChange,
  onDiscard,
  onSubmit,
  onAddMapper,
  onAddCommonProfile,
  onDeleteMapper,
}: ClientSamlTabProps) {
  const { confirm, ask, close } = useConfirmDeleteAlert()
  const isCustomSource = mapperDraft.source === CUSTOM_ATTRIBUTE_SOURCE

  if (isLoading) {
    return (
      <div className={cn(tokens.surface.panel, 'space-y-3 p-4')}>
        <div className='h-3 w-40 animate-pulse rounded bg-neutral-100' />
        <div className='h-8 w-full animate-pulse rounded bg-neutral-100' />
        <div className='h-8 w-full animate-pulse rounded bg-neutral-100' />
      </div>
    )
  }

  const nameIdDescription = NAME_ID_FORMAT_OPTIONS.find(
    (o) => o.value === draft.nameIdFormat
  )?.description

  const askDeleteMapper = (mapper: SamlAttributeMapper) =>
    ask({
      title: `Stop sending ${mapper.name}?`,
      description:
        'The application will no longer receive this attribute on the next sign-in. Anything relying on it may break.',
      onConfirm: () => {
        onDeleteMapper(mapper.id)
        close()
      },
    })

  return (
    <>
      {!isConfigured && (
        <div className='rounded-sm border border-fk-info-border bg-fk-info-soft/50 px-4 py-3'>
          <p className='text-sm font-medium text-neutral-900'>
            This client does not use SAML yet
          </p>
          <p className='mt-0.5 text-xs text-neutral-600'>
            Fill in the two values the application shows on its SAML settings page, then save.
            FerrisKey will start answering SAML sign-in requests for it.
          </p>
        </div>
      )}

      <Section
        title='Service provider'
        description='Identifies the application and tells FerrisKey where to send the assertion.'
      >
        <FieldRow
          label='Entity ID'
          description='The unique identifier the application publishes for itself, for example https://chat.acme.com/saml/sp/1.'
          htmlFor='saml-sp-entity-id'
        >
          <Input
            id='saml-sp-entity-id'
            value={draft.spEntityId}
            onChange={(e) => onDraftChange({ spEntityId: e.target.value })}
            className='max-w-lg font-mono-ui'
            aria-invalid={Boolean(errors.spEntityId)}
          />
          {errors.spEntityId && <p className='mt-1.5 text-xs text-fk-danger'>{errors.spEntityId}</p>}
        </FieldRow>

        <FieldRow
          label='Assertion Consumer Service URL'
          description='Where the signed assertion is posted after the user signs in.'
          htmlFor='saml-acs-url'
        >
          <Input
            id='saml-acs-url'
            value={draft.acsUrl}
            onChange={(e) => onDraftChange({ acsUrl: e.target.value })}
            className='max-w-lg font-mono-ui'
            aria-invalid={Boolean(errors.acsUrl)}
          />
          {errors.acsUrl && <p className='mt-1.5 text-xs text-fk-danger'>{errors.acsUrl}</p>}
        </FieldRow>

        <FieldRow
          label='Name ID format'
          description='How the user is identified inside the assertion.'
        >
          <div className='max-w-lg'>
            <Select
              value={draft.nameIdFormat}
              onValueChange={(v) => onDraftChange({ nameIdFormat: v })}
            >
              <SelectTrigger className='w-full'>
                <SelectValue placeholder='Select a name ID format' />
              </SelectTrigger>
              <SelectContent>
                {NAME_ID_FORMAT_OPTIONS.map((option) => (
                  <SelectItem key={option.value} value={option.value}>
                    {option.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            {nameIdDescription && (
              <p className='mt-1.5 text-xs text-neutral-500'>{nameIdDescription}</p>
            )}
            {errors.nameIdFormat && (
              <p className='mt-1.5 text-xs text-fk-danger'>{errors.nameIdFormat}</p>
            )}
          </div>
        </FieldRow>
      </Section>

      <Section title='Signatures' description='What FerrisKey signs, and what it demands signed.'>
        {SIGNATURE_FIELDS.map((field) => (
          <FieldRow key={field.key} label={field.label} description={field.description}>
            <SwitchField
              checked={draft[field.key]}
              onCheckedChange={(v) => onDraftChange({ [field.key]: v })}
            />
          </FieldRow>
        ))}
      </Section>

      {!isConfigured && (
        <div>
          <Button type='button' disabled={isSaving} onClick={onSubmit}>
            Enable SAML
          </Button>
        </div>
      )}

      {isConfigured && (
        <Section
          title='Attribute mappings'
          description='User details sent alongside the assertion. Match the attribute names the application asks for.'
          contained={false}
        >
          <div className='space-y-3'>
            {mappers.length === 0 ? (
              <div className='space-y-3 rounded-lg border border-dashed border-fk-line px-4 py-3'>
                <p className='text-xs text-neutral-500'>
                  No attributes are sent yet. Most applications need at least an email address.
                </p>
                <Button
                  type='button'
                  variant='outline'
                  size='sm'
                  disabled={isCreatingMapper}
                  onClick={onAddCommonProfile}
                >
                  Add email, first_name and last_name
                </Button>
              </div>
            ) : (
              <ul className={cn(tokens.surface.panel, 'px-3', tokens.surface.divider)}>
                {mappers.map((mapper) => (
                  <li key={mapper.id} className='flex items-center gap-3 py-2.5'>
                    <span className='shrink-0 font-mono-ui text-xs text-neutral-900'>
                      {mapper.name}
                    </span>
                    <span className='shrink-0 text-[11px] text-neutral-400'>from</span>
                    <span className='min-w-0 flex-1 truncate text-xs text-neutral-600'>
                      {describeAttributeSource(mapper.source)}
                    </span>
                    <span className='shrink-0 text-[11px] text-neutral-400'>
                      {describeAttributeNameFormat(mapper.name_format)}
                    </span>
                    <Button
                      variant='ghost'
                      size='icon'
                      disabled={isDeletingMapper}
                      aria-label={`Remove the ${mapper.name} attribute mapping`}
                      onClick={() => askDeleteMapper(mapper)}
                      className='size-7 text-neutral-400 hover:text-fk-danger'
                    >
                      <Trash2 />
                    </Button>
                  </li>
                ))}
              </ul>
            )}

            <div className={cn(tokens.surface.panel, 'grid gap-3 p-4 md:grid-cols-2')}>
              <div>
                <label
                  className='block pb-1 text-xs font-medium text-neutral-900'
                  htmlFor='saml-mapper-name'
                >
                  Attribute name
                </label>
                <Input
                  id='saml-mapper-name'
                  value={mapperDraft.name}
                  onChange={(e) => onMapperDraftChange({ name: e.target.value })}
                  className='font-mono-ui'
                  aria-invalid={Boolean(mapperErrors.name)}
                />
                {mapperErrors.name && (
                  <p className='mt-1.5 text-xs text-fk-danger'>{mapperErrors.name}</p>
                )}
              </div>

              <div>
                <label className='block pb-1 text-xs font-medium text-neutral-900'>
                  User detail to send
                </label>
                <Select
                  value={mapperDraft.source}
                  onValueChange={(v) => onMapperDraftChange({ source: v })}
                >
                  <SelectTrigger className='w-full'>
                    <SelectValue placeholder='Select a user detail' />
                  </SelectTrigger>
                  <SelectContent>
                    {BUILT_IN_SOURCE_OPTIONS.map((option) => (
                      <SelectItem key={option.value} value={option.value}>
                        {option.label}
                      </SelectItem>
                    ))}
                    <SelectItem value={CUSTOM_ATTRIBUTE_SOURCE}>Custom user attribute</SelectItem>
                  </SelectContent>
                </Select>
              </div>

              {isCustomSource && (
                <div>
                  <label
                    className='block pb-1 text-xs font-medium text-neutral-900'
                    htmlFor='saml-mapper-custom-key'
                  >
                    Attribute key
                  </label>
                  <Input
                    id='saml-mapper-custom-key'
                    value={mapperDraft.customKey}
                    onChange={(e) => onMapperDraftChange({ customKey: e.target.value })}
                    className='font-mono-ui'
                    aria-invalid={Boolean(mapperErrors.customKey)}
                  />
                  {mapperErrors.customKey && (
                    <p className='mt-1.5 text-xs text-fk-danger'>{mapperErrors.customKey}</p>
                  )}
                </div>
              )}

              <div>
                <label className='block pb-1 text-xs font-medium text-neutral-900'>
                  Name format
                </label>
                <Select
                  value={mapperDraft.nameFormat}
                  onValueChange={(v) => onMapperDraftChange({ nameFormat: v })}
                >
                  <SelectTrigger className='w-full'>
                    <SelectValue placeholder='Select a name format' />
                  </SelectTrigger>
                  <SelectContent>
                    {ATTRIBUTE_NAME_FORMAT_OPTIONS.map((option) => (
                      <SelectItem key={option.value} value={option.value}>
                        {option.label}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>

              <div className='md:col-span-2'>
                <Button
                  type='button'
                  disabled={!canAddMapper || isCreatingMapper}
                  onClick={onAddMapper}
                >
                  Add attribute
                </Button>
              </div>
            </div>
          </div>
        </Section>
      )}

      {isConfigured && (
        <FloatingActionBar
          show={dirtyCount > 0}
          title={`${dirtyCount} unsaved change${dirtyCount > 1 ? 's' : ''}`}
          description='Save the SAML service provider settings for this client.'
          onCancel={onDiscard}
          cancelLabel='Discard'
          actions={[{ label: 'Save changes', onClick: onSubmit }]}
        />
      )}

      <ConfirmDeleteAlert
        title={confirm.title}
        description={confirm.description}
        open={confirm.open}
        onConfirm={confirm.onConfirm}
        onCancel={close}
      />
    </>
  )
}
