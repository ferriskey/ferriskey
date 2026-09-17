import { Trash2 } from 'lucide-react'
import { Trans, useTranslation } from 'react-i18next'
import { Button } from '@/components/kit/button'
import { Input } from '@/components/ui/input'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import SaveBar from '@/components/kit/save-bar'
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
  { key: 'signAssertions', field: 'sign_assertions' },
  { key: 'signDocuments', field: 'sign_documents' },
  { key: 'wantAuthnRequestsSigned', field: 'want_authn_requests_signed' },
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
  const { t } = useTranslation('client')
  const { confirm, ask, close } = useConfirmDeleteAlert()
  const isCustomSource = mapperDraft.source === CUSTOM_ATTRIBUTE_SOURCE

  if (isLoading) {
    return (
      <div className={cn(tokens.surface.panel, 'space-y-3 p-4')}>
        <div className='h-3 w-40 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
        <div className='h-8 w-full animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
        <div className='h-8 w-full animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
      </div>
    )
  }

  const nameIdDescription = NAME_ID_FORMAT_OPTIONS.find(
    (o) => o.value === draft.nameIdFormat
  )?.description

  const askDeleteMapper = (mapper: SamlAttributeMapper) =>
    ask({
      title: t('saml.mappers.delete.title', { name: mapper.name }),
      description: t('saml.mappers.delete.description'),
      onConfirm: () => {
        onDeleteMapper(mapper.id)
        close()
      },
    })

  return (
    <>
      {!isConfigured && (
        <div className='rounded-sm border border-fk-info-border bg-fk-info-soft/50 px-4 py-3'>
          <p className='text-sm font-medium text-neutral-900 dark:text-neutral-100'>
            {t('saml.not_configured.title')}
          </p>
          <p className='mt-0.5 text-xs text-neutral-600 dark:text-neutral-400'>
            {t('saml.not_configured.hint')}
          </p>
        </div>
      )}

      <Section
        title={t('saml.service_provider.title')}
        description={t('saml.service_provider.description')}
      >
        <FieldRow
          label={t('saml.service_provider.entity_id.label')}
          description={t('saml.service_provider.entity_id.description')}
          htmlFor='saml-sp-entity-id'
        >
          <Input
            id='saml-sp-entity-id'
            value={draft.spEntityId}
            onChange={(e) => onDraftChange({ spEntityId: e.target.value })}
            className='max-w-lg'
            aria-invalid={Boolean(errors.spEntityId)}
          />
          {errors.spEntityId && <p className='mt-1.5 text-fk-danger text-xs'>{errors.spEntityId}</p>}
        </FieldRow>

        <FieldRow
          label={t('saml.service_provider.acs_url.label')}
          description={t('saml.service_provider.acs_url.description')}
          htmlFor='saml-acs-url'
        >
          <Input
            id='saml-acs-url'
            value={draft.acsUrl}
            onChange={(e) => onDraftChange({ acsUrl: e.target.value })}
            className='max-w-lg'
            aria-invalid={Boolean(errors.acsUrl)}
          />
          {errors.acsUrl && <p className='mt-1.5 text-fk-danger text-xs'>{errors.acsUrl}</p>}
        </FieldRow>

        <FieldRow
          label={t('saml.service_provider.name_id_format.label')}
          description={t('saml.service_provider.name_id_format.description')}
        >
          <div className='max-w-lg'>
            <Select
              value={draft.nameIdFormat}
              onValueChange={(v) => onDraftChange({ nameIdFormat: v })}
            >
              <SelectTrigger className='w-full'>
                <SelectValue placeholder={t('saml.service_provider.name_id_format.placeholder')} />
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
              <p className='mt-1.5 text-xs text-neutral-500 dark:text-neutral-400'>{nameIdDescription}</p>
            )}
            {errors.nameIdFormat && (
              <p className='mt-1.5 text-xs text-fk-danger'>{errors.nameIdFormat}</p>
            )}
          </div>
        </FieldRow>
      </Section>

      <Section title={t('saml.signatures.title')} description={t('saml.signatures.description')}>
        {SIGNATURE_FIELDS.map((signature) => (
          <FieldRow
            key={signature.key}
            label={t(`saml.signatures.${signature.field}.label`)}
            description={t(`saml.signatures.${signature.field}.description`)}
          >
            <SwitchField
              checked={draft[signature.key]}
              onCheckedChange={(v) => onDraftChange({ [signature.key]: v })}
            />
          </FieldRow>
        ))}
      </Section>

      {!isConfigured && (
        <div>
          <Button type='button' disabled={isSaving} onClick={onSubmit}>
            {t('saml.enable')}
          </Button>
        </div>
      )}

      {isConfigured && (
        <Section
          title={t('saml.mappers.title')}
          description={t('saml.mappers.description')}
          contained={false}
        >
          <div className='space-y-3'>
            {mappers.length === 0 ? (
              <div className='space-y-3 rounded-lg border border-dashed border-fk-line px-4 py-3'>
                <p className='text-xs text-neutral-500 dark:text-neutral-400'>
                  {t('saml.mappers.empty')}
                </p>
                <Button
                  type='button'
                  variant='outline'
                  size='sm'
                  disabled={isCreatingMapper}
                  onClick={onAddCommonProfile}
                >
                  {t('saml.mappers.add_common_profile')}
                </Button>
              </div>
            ) : (
              <ul className={cn(tokens.surface.panel, 'px-3', tokens.surface.divider)}>
                {mappers.map((mapper) => (
                  <li key={mapper.id} className='flex items-center gap-3 py-2.5'>
                    <Trans
                      i18nKey='client:saml.mappers.row'
                      values={{
                        name: mapper.name,
                        source: describeAttributeSource(mapper.source),
                      }}
                      components={{
                        attr: (
                          <span className='shrink-0 font-mono-ui text-xs text-neutral-900 dark:text-neutral-100' />
                        ),
                        from: (
                          <span className='shrink-0 text-[11px] text-neutral-400 dark:text-neutral-500' />
                        ),
                        detail: (
                          <span className='min-w-0 flex-1 truncate text-xs text-neutral-600 dark:text-neutral-400' />
                        ),
                      }}
                    />
                    <span className='shrink-0 text-[11px] text-neutral-400 dark:text-neutral-500'>
                      {describeAttributeNameFormat(mapper.name_format)}
                    </span>
                    <Button
                      variant='ghost'
                      size='icon'
                      disabled={isDeletingMapper}
                      aria-label={t('saml.mappers.remove', { name: mapper.name })}
                      onClick={() => askDeleteMapper(mapper)}
                      className='size-7 text-neutral-400 dark:text-neutral-500 hover:text-fk-danger'
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
                  className='block pb-1 text-xs font-medium text-neutral-900 dark:text-neutral-100'
                  htmlFor='saml-mapper-name'
                >
                  {t('saml.mappers.name.label')}
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
                <label className='block pb-1 text-xs font-medium text-neutral-900 dark:text-neutral-100'>
                  {t('saml.mappers.source.label')}
                </label>
                <Select
                  value={mapperDraft.source}
                  onValueChange={(v) => onMapperDraftChange({ source: v })}
                >
                  <SelectTrigger className='w-full'>
                    <SelectValue placeholder={t('saml.mappers.source.placeholder')} />
                  </SelectTrigger>
                  <SelectContent>
                    {BUILT_IN_SOURCE_OPTIONS.map((option) => (
                      <SelectItem key={option.value} value={option.value}>
                        {option.label}
                      </SelectItem>
                    ))}
                    <SelectItem value={CUSTOM_ATTRIBUTE_SOURCE}>
                      {t('saml.mappers.source.custom')}
                    </SelectItem>
                  </SelectContent>
                </Select>
              </div>

              {isCustomSource && (
                <div>
                  <label
                    className='block pb-1 text-xs font-medium text-neutral-900 dark:text-neutral-100'
                    htmlFor='saml-mapper-custom-key'
                  >
                    {t('saml.mappers.custom_key.label')}
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
                <label className='block pb-1 text-xs font-medium text-neutral-900 dark:text-neutral-100'>
                  {t('saml.mappers.name_format.label')}
                </label>
                <Select
                  value={mapperDraft.nameFormat}
                  onValueChange={(v) => onMapperDraftChange({ nameFormat: v })}
                >
                  <SelectTrigger className='w-full'>
                    <SelectValue placeholder={t('saml.mappers.name_format.placeholder')} />
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
                  {t('saml.mappers.submit')}
                </Button>
              </div>
            </div>
          </div>
        </Section>
      )}

      {isConfigured && (
        <SaveBar
          show={dirtyCount > 0}
          title={t('shared.unsaved', { count: dirtyCount })}
          description={t('saml.save.description')}
          onCancel={onDiscard}
          cancelLabel={t('shared.discard')}
          actions={[{ label: t('shared.save'), onClick: onSubmit }]}
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
