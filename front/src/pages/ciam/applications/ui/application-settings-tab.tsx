import { useTranslation } from 'react-i18next'
import { Input } from '@/components/ui/input'
import { DurationInput } from '@/components/ui/duration-input'
import SaveBar from '@/components/kit/save-bar'
import { DangerZone } from '@/components/kit/danger-zone'
import { ChipInput, ChoiceCards, FieldRow, Pill, Section, SwitchField } from '@/components/kit'
import type { Choice } from '@/components/kit'
import { Power, PowerOff } from 'lucide-react'
import { Schemas } from '@/api/api.client'
import {
  applicationFields,
  applicationTypeMeta,
  inferApplicationType,
  type ConsoleTranslate,
} from '../application-types'

import Client = Schemas.Client

type ApplicationState = 'enabled' | 'disabled'

const ENABLED_STATE: ApplicationState = 'enabled'
const DISABLED_STATE: ApplicationState = 'disabled'

const stateChoices = (t: ConsoleTranslate): Choice<ApplicationState>[] => [
  {
    value: 'enabled',
    label: t('applications.settings.state.enabled.label'),
    description: t('applications.settings.state.enabled.description'),
    icon: Power,
  },
  {
    value: 'disabled',
    label: t('applications.settings.state.disabled.label'),
    description: t('applications.settings.state.disabled.description'),
    icon: PowerOff,
  },
]

export interface ApplicationSettingsDraft {
  name: string
  enabled: boolean
  directAccessGrants: boolean
  deviceCodeGrant: boolean
  requirePkce: boolean
  accessTokenLifetime: number | null
  refreshTokenLifetime: number | null
  idTokenLifetime: number | null
  temporaryTokenLifetime: number | null
}

export interface ApplicationSettingsTabProps {
  application: Client
  draft: ApplicationSettingsDraft
  errors: { name?: string }
  dirtyCount: number
  callbacks: string[]
  callbackError?: string
  origins: string[]
  originError?: string
  onDraftChange: (patch: Partial<ApplicationSettingsDraft>) => void
  onCallbacksChange: (next: string[]) => void
  onOriginsChange: (next: string[]) => void
  onDiscard: () => void
  onSave: () => void
  onDelete: () => void
}

const LIFETIMES = [
  { key: 'accessTokenLifetime', catalog: 'access_token' },
  { key: 'refreshTokenLifetime', catalog: 'refresh_token' },
  { key: 'idTokenLifetime', catalog: 'id_token' },
  { key: 'temporaryTokenLifetime', catalog: 'temporary_token' },
] as const

export default function ApplicationSettingsTab({
  application,
  draft,
  errors,
  dirtyCount,
  callbacks,
  callbackError,
  origins,
  originError,
  onDraftChange,
  onCallbacksChange,
  onOriginsChange,
  onDiscard,
  onSave,
  onDelete,
}: ApplicationSettingsTabProps) {
  const { t } = useTranslation('console')

  const type = inferApplicationType(application)
  const meta = applicationTypeMeta(type, t)
  const fields = applicationFields(type, t)

  const pkceDescription = meta.holdsSecret
    ? t('applications.settings.methods.pkce_description_confidential')
    : t('applications.settings.methods.pkce_description_public')

  return (
    <>
      <Section
        title={t('applications.settings.general.title')}
        description={t('applications.settings.general.description')}
      >
        <FieldRow
          label={t('applications.settings.general.name_label')}
          description={t('applications.settings.general.name_description')}
          htmlFor='application-name'
        >
          <Input
            id='application-name'
            value={draft.name}
            onChange={(e) => onDraftChange({ name: e.target.value })}
            className='max-w-sm'
            aria-invalid={Boolean(errors.name)}
          />
          {errors.name && <p className='mt-1.5 text-xs text-fk-danger'>{errors.name}</p>}
        </FieldRow>

        <FieldRow
          label={t('applications.settings.general.client_id_label')}
          description={t('applications.settings.general.client_id_description')}
        >
          <p className='font-mono-ui text-xs text-neutral-500 dark:text-neutral-400'>
            {application.client_id}
          </p>
        </FieldRow>

        <FieldRow
          label={t('applications.settings.general.type_label')}
          description={t('applications.settings.general.type_description')}
        >
          <div className='flex flex-wrap items-center gap-2'>
            <Pill tone={meta.tone}>{meta.label}</Pill>
            <span className='font-mono-ui text-xs text-neutral-500 dark:text-neutral-400'>
              {meta.flow}
            </span>
          </div>
        </FieldRow>

        <FieldRow
          label={t('applications.settings.general.availability_label')}
          description={t('applications.settings.general.availability_description')}
        >
          <ChoiceCards
            label={t('applications.settings.general.state_picker_label')}
            value={draft.enabled ? ENABLED_STATE : DISABLED_STATE}
            onChange={(v: ApplicationState) => onDraftChange({ enabled: v === ENABLED_STATE })}
            options={stateChoices(t)}
          />
        </FieldRow>
      </Section>

      {meta.usesAuthorizationCode && (
        <Section
          title={t('applications.settings.urls.title')}
          description={t('applications.settings.urls.description')}
        >
          <FieldRow label={fields.callbacksLabel} description={fields.callbacksHint}>
            <ChipInput
              values={callbacks}
              onChange={onCallbacksChange}
              placeholder={fields.callbacksPlaceholder}
              emptyHint={t('applications.fields.empty.callbacks_required')}
            />
            {callbackError && <p className='mt-1.5 text-xs text-fk-danger'>{callbackError}</p>}
          </FieldRow>

          <FieldRow
            label={t('applications.settings.urls.origins_label')}
            description={t('applications.settings.urls.origins_description')}
          >
            <ChipInput
              values={origins}
              onChange={onOriginsChange}
              placeholder={t('applications.settings.urls.origins_placeholder')}
              emptyHint={t('applications.fields.empty.origins')}
            />
            {originError && <p className='mt-1.5 text-xs text-fk-danger'>{originError}</p>}
          </FieldRow>
        </Section>
      )}

      <Section
        title={t('applications.settings.methods.title')}
        description={t('applications.settings.methods.description')}
      >
        <FieldRow
          label={t('applications.settings.methods.password_label')}
          description={t('applications.settings.methods.password_description')}
        >
          <SwitchField
            checked={draft.directAccessGrants}
            onCheckedChange={(v) => onDraftChange({ directAccessGrants: v })}
          />
        </FieldRow>

        <FieldRow
          label={t('applications.settings.methods.device_label')}
          description={t('applications.settings.methods.device_description')}
        >
          <SwitchField
            checked={draft.deviceCodeGrant}
            onCheckedChange={(v) => onDraftChange({ deviceCodeGrant: v })}
          />
        </FieldRow>

        {meta.usesAuthorizationCode && (
          <FieldRow
            label={t('applications.settings.methods.pkce_label')}
            description={pkceDescription}
          >
            <SwitchField
              checked={draft.requirePkce}
              onCheckedChange={(v) => onDraftChange({ requirePkce: v })}
              onLabel={t('applications.settings.methods.pkce_on')}
              offLabel={t('applications.settings.methods.pkce_off')}
            />
          </FieldRow>
        )}
      </Section>

      <Section
        title={t('applications.settings.lifetimes.title')}
        description={t('applications.settings.lifetimes.description')}
      >
        {LIFETIMES.map((lifetime) => {
          const label = t(`applications.settings.lifetimes.${lifetime.catalog}.label`)

          return (
            <FieldRow
              key={lifetime.key}
              label={label}
              description={t(`applications.settings.lifetimes.${lifetime.catalog}.description`)}
            >
              <DurationInput
                label={label}
                value={draft[lifetime.key]}
                onChange={(seconds) => onDraftChange({ [lifetime.key]: seconds })}
                nullable
              />
            </FieldRow>
          )
        })}
      </Section>

      <DangerZone
        resourceName={application.name || application.client_id}
        label={t('applications.settings.danger.label')}
        description={t('applications.settings.danger.description')}
        buttonLabel={t('applications.settings.danger.button')}
        confirmTitle={t('applications.settings.danger.confirm_title')}
        confirmDescription={t('applications.settings.danger.confirm_description', {
          name: application.name || application.client_id,
        })}
        onConfirm={onDelete}
      />

      <SaveBar
        show={dirtyCount > 0}
        title={t('applications.settings.save_bar.title', { count: dirtyCount })}
        description={t('applications.settings.save_bar.description')}
        onCancel={onDiscard}
        cancelLabel={t('applications.settings.save_bar.cancel')}
        actions={[{ label: t('applications.settings.save_bar.submit'), onClick: onSave }]}
      />
    </>
  )
}
