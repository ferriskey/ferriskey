import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Eye, EyeOff, Link2, Power, PowerOff, RefreshCcw, ShieldCheck } from 'lucide-react'
import { Input } from '@/components/ui/input'
import {
  InputGroup,
  InputGroupAddon,
  InputGroupButton,
  InputGroupInput,
} from '@/components/ui/input-group'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { DurationInput } from '@/components/ui/duration-input'
import { ChoiceCards, FieldRow, Section, SwitchField, type Choice } from '@/components/kit'
import {
  MASKED_SECRET,
  PRIORITY_HINT_KEY,
  PRIORITY_LABEL_KEY,
  PRIORITY_ORDER,
  USER_FEDERATION_NAMESPACE,
  type LdapSettings,
  type ProviderPriority,
  type SyncMode,
} from '../provider-config'

export interface ProviderErrors {
  name?: string
  connectionUrl?: string
  baseDn?: string
  userSearchFilter?: string
  syncInterval?: string
  bindPassword?: string
}

const CONNECTION_URL_PLACEHOLDER = 'ldaps://ldap.example.com:636'
const BASE_DN_PLACEHOLDER = 'dc=example,dc=com'
const BIND_DN_PLACEHOLDER = 'cn=ferriskey,ou=services,dc=example,dc=com'

type EnabledState = 'enabled' | 'disabled'

const ENABLED_STATE: EnabledState = 'enabled'
const DISABLED_STATE: EnabledState = 'disabled'

const ENABLED_CHOICES = [
  { value: ENABLED_STATE, labelKey: 'form.enabled.choices.enabled', icon: Power },
  { value: DISABLED_STATE, labelKey: 'form.enabled.choices.disabled', icon: PowerOff },
] as const

const SYNC_MODE_CHOICES = [
  { value: 'Import', labelKey: 'form.sync.mode.choices.import', icon: RefreshCcw },
  { value: 'Force', labelKey: 'form.sync.mode.choices.force', icon: ShieldCheck },
  { value: 'LinkOnly', labelKey: 'form.sync.mode.choices.link_only', icon: Link2 },
] as const

export function BasicSettingsFields({
  name,
  enabled,
  priority,
  errors,
  onNameChange,
  onEnabledChange,
  onPriorityChange,
  children,
}: {
  name: string
  enabled: boolean
  priority: ProviderPriority
  errors: ProviderErrors
  onNameChange: (v: string) => void
  onEnabledChange: (v: boolean) => void
  onPriorityChange: (v: ProviderPriority) => void
  children?: React.ReactNode
}) {
  const { t } = useTranslation(USER_FEDERATION_NAMESPACE)

  const enabledChoices: Choice<EnabledState>[] = ENABLED_CHOICES.map((choice) => ({
    value: choice.value,
    label: t(`${choice.labelKey}.label`),
    description: t(`${choice.labelKey}.description`),
    icon: choice.icon,
  }))

  return (
    <Section
      title={t('form.basic.title')}
      description={t('form.basic.description')}
    >
      <FieldRow
        label={t('form.name.label')}
        description={t('form.name.description')}
        htmlFor='provider-name'
      >
        <Input
          id='provider-name'
          value={name}
          onChange={(e) => onNameChange(e.target.value)}
          className='max-w-sm'
          aria-invalid={Boolean(errors.name)}
        />
        {errors.name && <p className='mt-1.5 text-fk-danger text-xs'>{errors.name}</p>}
      </FieldRow>

      {children}

      <FieldRow
        label={t('form.priority.label')}
        description={t('form.priority.description')}
      >
        <div className='max-w-sm'>
          <Select
            value={priority}
            onValueChange={(v) => onPriorityChange(v as ProviderPriority)}
          >
            <SelectTrigger className='w-full'>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {PRIORITY_ORDER.map((p) => (
                <SelectItem key={p} value={p}>
                  {t(PRIORITY_LABEL_KEY[p])}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <p className='mt-1.5 text-xs text-neutral-500 dark:text-neutral-400'>
            {t(PRIORITY_HINT_KEY[priority])}
          </p>
        </div>
      </FieldRow>

      <FieldRow
        label={t('form.enabled.label')}
        description={t('form.enabled.description')}
      >
        <ChoiceCards
          label={t('form.enabled.picker_label')}
          value={enabled ? ENABLED_STATE : DISABLED_STATE}
          onChange={(v) => onEnabledChange(v === ENABLED_STATE)}
          options={enabledChoices}
        />
      </FieldRow>
    </Section>
  )
}

export function LdapConnectionFields({
  settings,
  bindPassword,
  secretStored,
  secretRequired,
  errors,
  onChange,
  onBindPasswordChange,
}: {
  settings: LdapSettings
  bindPassword: string
  secretStored: boolean
  secretRequired: boolean
  errors: ProviderErrors
  onChange: (patch: Partial<LdapSettings>) => void
  onBindPasswordChange: (v: string) => void
}) {
  const { t } = useTranslation(USER_FEDERATION_NAMESPACE)
  const [reveal, setReveal] = useState(false)

  return (
    <>
      <Section
        title={t('form.connection.title')}
        description={t('form.connection.description')}
      >
        <FieldRow
          label={t('form.connection_url.label')}
          description={t('form.connection_url.description')}
          htmlFor='provider-connection-url'
        >
          <Input
            id='provider-connection-url'
            value={settings.connectionUrl}
            onChange={(e) => onChange({ connectionUrl: e.target.value })}
            placeholder={CONNECTION_URL_PLACEHOLDER}
            className='max-w-lg'
            aria-invalid={Boolean(errors.connectionUrl)}
          />
          {errors.connectionUrl && (
            <p className='mt-1.5 text-xs text-fk-danger'>{errors.connectionUrl}</p>
          )}
        </FieldRow>

        <FieldRow
          label={t('form.base_dn.label')}
          description={t('form.base_dn.description')}
          htmlFor='provider-base-dn'
        >
          <Input
            id='provider-base-dn'
            value={settings.baseDn}
            onChange={(e) => onChange({ baseDn: e.target.value })}
            placeholder={BASE_DN_PLACEHOLDER}
            className='max-w-lg'
            aria-invalid={Boolean(errors.baseDn)}
          />
          {errors.baseDn && <p className='mt-1.5 text-xs text-fk-danger'>{errors.baseDn}</p>}
        </FieldRow>

        <FieldRow
          label={t('form.bind_dn.label')}
          description={t('form.bind_dn.description')}
          htmlFor='provider-bind-dn'
        >
          <Input
            id='provider-bind-dn'
            value={settings.bindDn}
            onChange={(e) => onChange({ bindDn: e.target.value })}
            placeholder={BIND_DN_PLACEHOLDER}
            className='max-w-lg'
          />
        </FieldRow>

        <FieldRow
          label={t('form.bind_password.label')}
          description={
            secretStored
              ? t('form.bind_password.description.stored')
              : t('form.bind_password.description.empty')
          }
          htmlFor='provider-bind-password'
        >
          {secretStored && (
            <p className='mb-2 font-mono-ui text-xs text-neutral-400 dark:text-neutral-500'>
              {t('form.bind_password.stored_hint', { mask: MASKED_SECRET })}
            </p>
          )}
          <InputGroup className='max-w-sm'>
            <InputGroupInput
              id='provider-bind-password'
              type={reveal ? 'text' : 'password'}
              value={bindPassword}
              onChange={(e) => onBindPasswordChange(e.target.value)}
              className='font-mono-ui'
              autoComplete='new-password'
            />
            <InputGroupAddon align='inline-end'>
              <InputGroupButton
                size='icon-xs'
                type='button'
                aria-label={
                  reveal ? t('form.bind_password.hide') : t('form.bind_password.reveal')
                }
                onClick={() => setReveal((v) => !v)}
              >
                {reveal ? <EyeOff /> : <Eye />}
              </InputGroupButton>
            </InputGroupAddon>
          </InputGroup>
          {secretRequired && !bindPassword && (
            <p className='mt-1.5 text-xs text-fk-danger'>{t('form.bind_password.required')}</p>
          )}
        </FieldRow>

        <FieldRow label={t('form.use_tls.label')} description={t('form.use_tls.description')}>
          <SwitchField
            checked={settings.useTls}
            onCheckedChange={(v) => onChange({ useTls: v })}
          />
        </FieldRow>
      </Section>

      <Section
        title={t('form.search.title')}
        description={t('form.search.description')}
      >
        <FieldRow
          label={t('form.search_filter.label')}
          description={t('form.search_filter.description')}
          htmlFor='provider-search-filter'
        >
          <Input
            id='provider-search-filter'
            value={settings.userSearchFilter}
            onChange={(e) => onChange({ userSearchFilter: e.target.value })}
            className='max-w-lg'
            aria-invalid={Boolean(errors.userSearchFilter)}
          />
          {errors.userSearchFilter && (
            <p className='mt-1.5 text-xs text-fk-danger'>{errors.userSearchFilter}</p>
          )}
        </FieldRow>
      </Section>
    </>
  )
}

export function SynchronisationFields({
  syncEnabled,
  syncMode,
  syncIntervalSeconds,
  errors,
  onSyncEnabledChange,
  onSyncModeChange,
  onSyncIntervalChange,
}: {
  syncEnabled: boolean
  syncMode: SyncMode
  syncIntervalSeconds: number
  errors: ProviderErrors
  onSyncEnabledChange: (v: boolean) => void
  onSyncModeChange: (v: SyncMode) => void
  onSyncIntervalChange: (seconds: number) => void
}) {
  const { t } = useTranslation(USER_FEDERATION_NAMESPACE)

  const syncModeChoices: Choice<SyncMode>[] = SYNC_MODE_CHOICES.map((choice) => ({
    value: choice.value,
    label: t(`${choice.labelKey}.label`),
    description: t(`${choice.labelKey}.description`),
    icon: choice.icon,
  }))

  return (
    <Section
      title={t('form.sync.title')}
      description={t('form.sync.description')}
    >
      <FieldRow
        label={t('form.sync.scheduled.label')}
        description={t('form.sync.scheduled.description')}
      >
        <SwitchField checked={syncEnabled} onCheckedChange={onSyncEnabledChange} />
      </FieldRow>

      <FieldRow
        label={t('form.sync.mode.label')}
        description={t('form.sync.mode.description')}
      >
        <ChoiceCards
          label={t('form.sync.mode.picker_label')}
          value={syncMode}
          onChange={onSyncModeChange}
          options={syncModeChoices}
          className='grid-cols-3'
        />
      </FieldRow>

      <FieldRow
        label={t('form.sync.interval.label')}
        description={t('form.sync.interval.description')}
      >
        <div className='max-w-xs'>
          <DurationInput
            label={t('form.sync.interval.input_label')}
            value={syncIntervalSeconds}
            onChange={(seconds) => onSyncIntervalChange(seconds ?? 0)}
            error={errors.syncInterval}
          />
        </div>
      </FieldRow>
    </Section>
  )
}
