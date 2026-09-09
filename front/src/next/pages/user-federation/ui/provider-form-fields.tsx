import { useState } from 'react'
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
  PRIORITY_ORDER,
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

const enabledChoices: Choice<'enabled' | 'disabled'>[] = [
  {
    value: 'enabled',
    label: 'Enabled',
    description: 'The provider takes part in authentication.',
    icon: Power,
  },
  {
    value: 'disabled',
    label: 'Disabled',
    description: 'Ignored, but the accounts already imported stay linked.',
    icon: PowerOff,
  },
]

const syncModeChoices: Choice<SyncMode>[] = [
  {
    value: 'Import',
    label: 'Import',
    description: 'Creates the missing accounts and updates the existing ones.',
    icon: RefreshCcw,
  },
  {
    value: 'Force',
    label: 'Force',
    description: 'Also disables the local accounts the directory no longer returns.',
    icon: ShieldCheck,
  },
  {
    value: 'LinkOnly',
    label: 'Link only',
    description: 'Links accounts already present, never creates one.',
    icon: Link2,
  },
]

const priorityHints: Record<ProviderPriority, string> = {
  Primary: 'Queried first.',
  Secondary: 'Queried when the primary does not answer.',
  Development: 'Working environment.',
  Legacy: 'Kept for the duration of a migration.',
}

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
  return (
    <Section
      title='Basic Settings'
      description='Who the provider is, and its place in the query order.'
    >
      <FieldRow
        label='Provider name'
        description='Names the provider in this console; it must be unique in the realm.'
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
        label='Priority'
        description='Order in which the providers are queried when an account is looked up.'
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
                  {p}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <p className='mt-1.5 text-xs text-neutral-500 dark:text-neutral-400'>{priorityHints[priority]}</p>
        </div>
      </FieldRow>

      <FieldRow
        label='Enabled'
        description='A disabled provider is no longer queried and no longer synchronises.'
      >
        <ChoiceCards
          label='Provider state'
          value={enabled ? 'enabled' : 'disabled'}
          onChange={(v) => onEnabledChange(v === 'enabled')}
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
  const [reveal, setReveal] = useState(false)

  return (
    <>
      <Section
        title='Connection Settings'
        description='How the directory is reached, and the service account used to read it.'
      >
        <FieldRow
          label='Connection URL'
          description='Without a scheme, ldaps:// is assumed when TLS is on, ldap:// otherwise.'
          htmlFor='provider-connection-url'
        >
          <Input
            id='provider-connection-url'
            value={settings.connectionUrl}
            onChange={(e) => onChange({ connectionUrl: e.target.value })}
            placeholder='ldaps://ldap.example.com:636'
            className='max-w-lg'
            aria-invalid={Boolean(errors.connectionUrl)}
          />
          {errors.connectionUrl && (
            <p className='mt-1.5 text-xs text-fk-danger'>{errors.connectionUrl}</p>
          )}
        </FieldRow>

        <FieldRow
          label='Base DN'
          description='Root under which the accounts are searched.'
          htmlFor='provider-base-dn'
        >
          <Input
            id='provider-base-dn'
            value={settings.baseDn}
            onChange={(e) => onChange({ baseDn: e.target.value })}
            placeholder='dc=example,dc=com'
            className='max-w-lg'
            aria-invalid={Boolean(errors.baseDn)}
          />
          {errors.baseDn && <p className='mt-1.5 text-xs text-fk-danger'>{errors.baseDn}</p>}
        </FieldRow>

        <FieldRow
          label='Bind DN'
          description='Optional: left empty, the directory is read anonymously.'
          htmlFor='provider-bind-dn'
        >
          <Input
            id='provider-bind-dn'
            value={settings.bindDn}
            onChange={(e) => onChange({ bindDn: e.target.value })}
            placeholder='cn=ferriskey,ou=services,dc=example,dc=com'
            className='max-w-lg'
          />
        </FieldRow>

        <FieldRow
          label='Bind credential'
          description={
            secretStored
              ? 'Stored encrypted and never returned. Type a new one to replace it.'
              : 'Stored encrypted and never returned once saved.'
          }
          htmlFor='provider-bind-password'
        >
          {secretStored && (
            <p className='mb-2 font-mono-ui text-xs text-neutral-400 dark:text-neutral-500'>
              ******** · saved, not readable
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
                aria-label={reveal ? 'Hide the credential' : 'Show the credential'}
                onClick={() => setReveal((v) => !v)}
              >
                {reveal ? <EyeOff /> : <Eye />}
              </InputGroupButton>
            </InputGroupAddon>
          </InputGroup>
          {secretRequired && !bindPassword && (
            <p className='mt-1.5 text-xs text-fk-danger'>
              Changing the connection rewrites the whole configuration, and the saved
              credential cannot be read back — type it again to save.
            </p>
          )}
        </FieldRow>

        <FieldRow label='Use TLS' description='Encrypts the connection to the directory.'>
          <SwitchField
            checked={settings.useTls}
            onCheckedChange={(v) => onChange({ useTls: v })}
          />
        </FieldRow>
      </Section>

      <Section
        title='User Search Settings'
        description='How an account is found in the directory.'
      >
        <FieldRow
          label='User search filter'
          description='LDAP filter applied to the lookup.'
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
  return (
    <Section
      title='Synchronization Settings'
      description='When the directory is walked again, and what it does to the accounts.'
    >
      <FieldRow
        label='Scheduled synchronisation'
        description='Turned off, the provider only synchronises when asked from this page.'
      >
        <SwitchField checked={syncEnabled} onCheckedChange={onSyncEnabledChange} />
      </FieldRow>

      <FieldRow
        label='Sync mode'
        description='What a scheduled run does with the accounts it meets. A run started by hand always imports.'
      >
        <ChoiceCards
          label='Synchronisation mode'
          value={syncMode}
          onChange={onSyncModeChange}
          options={syncModeChoices}
          className='grid-cols-3'
        />
      </FieldRow>

      <FieldRow
        label='Sync interval'
        description='Time between two runs. The server keeps whole minutes, 60 seconds at the least.'
      >
        <div className='max-w-xs'>
          <DurationInput
            label='Interval'
            value={syncIntervalSeconds}
            onChange={(seconds) => onSyncIntervalChange(seconds ?? 0)}
            error={errors.syncInterval}
          />
        </div>
      </FieldRow>
    </Section>
  )
}
