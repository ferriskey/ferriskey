import { ArrowLeft } from 'lucide-react'
import { Button } from '@/components/kit/button'
import SaveBar from '@/components/kit/save-bar'
import { Pill } from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import {
  type LdapSettings,
  type ProviderPriority,
  type SyncMode,
} from '../provider-config'
import {
  BasicSettingsFields,
  LdapConnectionFields,
  SynchronisationFields,
  type ProviderErrors,
} from './provider-form-fields'

export interface PageCreateProviderProps {
  name: string
  enabled: boolean
  priority: ProviderPriority
  ldap: LdapSettings
  bindPassword: string
  syncEnabled: boolean
  syncMode: SyncMode
  syncIntervalSeconds: number
  errors: ProviderErrors
  canSubmit: boolean
  onNameChange: (v: string) => void
  onEnabledChange: (v: boolean) => void
  onPriorityChange: (v: ProviderPriority) => void
  onLdapChange: (patch: Partial<LdapSettings>) => void
  onBindPasswordChange: (v: string) => void
  onSyncEnabledChange: (v: boolean) => void
  onSyncModeChange: (v: SyncMode) => void
  onSyncIntervalChange: (seconds: number) => void
  onBack: () => void
  onChangeKind: () => void
  onSubmit: () => void
}

export default function PageCreateProvider({
  name,
  enabled,
  priority,
  ldap,
  bindPassword,
  syncEnabled,
  syncMode,
  syncIntervalSeconds,
  errors,
  canSubmit,
  onNameChange,
  onEnabledChange,
  onPriorityChange,
  onLdapChange,
  onBindPasswordChange,
  onSyncEnabledChange,
  onSyncModeChange,
  onSyncIntervalChange,
  onBack,
  onChangeKind,
  onSubmit,
}: PageCreateProviderProps) {
  return (
    <div className={cn('mx-auto', tokens.page.maxWidth, tokens.page.padding)}>
      <Button variant='ghost' size='sm' className='-ml-2 mb-2 text-neutral-500' onClick={onBack}>
        <ArrowLeft className='size-3.5' />
        User Federation
      </Button>

      <div className='flex flex-wrap items-center gap-2 pb-3'>
        <h1 className={tokens.header.title}>New provider</h1>
        <Pill tone='violet' mono>
          Ldap
        </Pill>
        <button
          type='button'
          onClick={onChangeKind}
          className='cursor-pointer text-xs text-neutral-500 underline-offset-2 hover:text-fk-primary-text hover:underline'
        >
          change type
        </button>
      </div>

      <div className={tokens.page.blockGap}>
        <BasicSettingsFields
          name={name}
          enabled={enabled}
          priority={priority}
          errors={errors}
          onNameChange={onNameChange}
          onEnabledChange={onEnabledChange}
          onPriorityChange={onPriorityChange}
        />

        <LdapConnectionFields
          settings={ldap}
          bindPassword={bindPassword}
          secretStored={false}
          secretRequired={false}
          errors={errors}
          onChange={onLdapChange}
          onBindPasswordChange={onBindPasswordChange}
        />

        <SynchronisationFields
          syncEnabled={syncEnabled}
          syncMode={syncMode}
          syncIntervalSeconds={syncIntervalSeconds}
          errors={errors}
          onSyncEnabledChange={onSyncEnabledChange}
          onSyncModeChange={onSyncModeChange}
          onSyncIntervalChange={onSyncIntervalChange}
        />
      </div>

      <SaveBar
        show={canSubmit}
        title='Create provider'
        description='Once created, the provider can be tested and synchronised from its page.'
        onCancel={onBack}
        actions={[{ label: 'Create provider', onClick: onSubmit }]}
      />
    </div>
  )
}
