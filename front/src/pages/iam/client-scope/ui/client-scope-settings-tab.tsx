import { useTranslation } from 'react-i18next'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { FieldRow, Section } from '@/components/kit'
import { DangerZone } from '@/components/kit/danger-zone'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import { Schemas } from '@/api/api.client'
import ScopeTypeHint from './scope-type-hint'
import type { ScopeTypeChoice } from './page-create-client-scope'
import { SCOPE_TYPE_CHOICES } from '../scope-type'

import ClientScope = Schemas.ClientScope

export interface ClientScopeSettingsTabProps {
  scope: ClientScope
  name: string
  description: string
  scopeType: ScopeTypeChoice
  nameError?: string
  onNameChange: (v: string) => void
  onDescriptionChange: (v: string) => void
  onScopeTypeChange: (v: ScopeTypeChoice) => void
  onDelete: () => void
}

export default function ClientScopeSettingsTab({
  scope,
  name,
  description,
  scopeType,
  nameError,
  onNameChange,
  onDescriptionChange,
  onScopeTypeChange,
  onDelete,
}: ClientScopeSettingsTabProps) {
  const { t } = useTranslation('client-scope')
  const attributes = scope.attributes ?? []

  return (
    <>
      <Section
        title={t('detail.settings.general.title')}
        description={t('detail.settings.general.description')}
      >
        <FieldRow
          label={t('scope_form.name.label')}
          description={t('scope_form.name.description')}
          htmlFor='scope-name'
        >
          <Input
            id='scope-name'
            value={name}
            onChange={(e) => onNameChange(e.target.value)}
            className='max-w-sm'
            aria-invalid={Boolean(nameError)}
          />
          {nameError && <p className='mt-1.5 text-fk-danger text-xs'>{nameError}</p>}
        </FieldRow>

        <FieldRow
          label={t('scope_form.description.label')}
          description={t('scope_form.description.description')}
          htmlFor='scope-description'
        >
          <Textarea
            id='scope-description'
            value={description}
            onChange={(e) => onDescriptionChange(e.target.value)}
            className='max-w-lg'
            rows={3}
          />
        </FieldRow>

        <FieldRow
          label={t('scope_form.protocol.label')}
          description={t('scope_form.protocol.description')}
          htmlFor='scope-protocol'
        >
          <Input
            id='scope-protocol'
            value={scope.protocol}
            disabled
            className='max-w-sm'
          />
        </FieldRow>

        <FieldRow
          label={t('scope_form.type.label')}
          description={t('scope_form.type.description')}
          htmlFor='scope-type'
        >
          <div className='max-w-sm'>
            <Select value={scopeType} onValueChange={(v) => onScopeTypeChange(v as ScopeTypeChoice)}>
              <SelectTrigger id='scope-type' className='w-full'>
                <SelectValue placeholder={t('scope_form.type.placeholder')} />
              </SelectTrigger>
              <SelectContent>
                {SCOPE_TYPE_CHOICES.map((choice) => (
                  <SelectItem key={choice.value} value={choice.value}>
                    {t(choice.labelKey)}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <ScopeTypeHint scopeType={scopeType} />
            {scope.default_scope_type === 'NONE' && (
              <p className='mt-1.5 text-xs text-fk-amber'>{t('detail.settings.none_warning')}</p>
            )}
          </div>
        </FieldRow>
      </Section>

      <Section
        title={t('detail.settings.attributes.title')}
        description={t('detail.settings.attributes.description')}
        contained={attributes.length > 0}
      >
        {attributes.length > 0 ? (
          <dl className={tokens.surface.divider}>
            {attributes.map((attribute) => (
              <div
                key={attribute.id}
                className='grid gap-x-6 py-2.5 md:grid-cols-[minmax(0,18rem)_minmax(0,1fr)]'
              >
                <dt className='font-mono-ui text-xs text-neutral-500 dark:text-neutral-400'>{attribute.name}</dt>
                <dd className='min-w-0 break-all font-mono-ui text-xs text-neutral-900 dark:text-neutral-100'>
                  {attribute.value || '—'}
                </dd>
              </div>
            ))}
          </dl>
        ) : (
          <p
            className={cn(
              'rounded-sm border border-dashed border-fk-line px-4 py-3 text-xs text-neutral-500 dark:text-neutral-400'
            )}
          >
            {t('detail.settings.attributes.empty')}
          </p>
        )}
      </Section>

      <DangerZone
        resourceName={scope.name}
        label={t('detail.settings.danger.label')}
        description={t('detail.settings.danger.description')}
        buttonLabel={t('detail.settings.danger.button')}
        confirmTitle={t('detail.settings.danger.confirm_title')}
        confirmDescription={t('detail.settings.danger.confirm_description', { name: scope.name })}
        onConfirm={onDelete}
      />
    </>
  )
}
