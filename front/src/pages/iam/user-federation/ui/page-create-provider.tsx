import { ArrowLeft } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/kit/button'
import SaveBar from '@/components/kit/save-bar'
import { PageShell, Pill } from '@/components/kit'
import { tokens } from '@/styles/style-tokens'
import {
  LDAP_PROVIDER_TYPE,
  USER_FEDERATION_NAMESPACE,
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
  const { t } = useTranslation(USER_FEDERATION_NAMESPACE)

  return (
    <PageShell>
      <Button variant='ghost' size='sm' className='-ml-2 mb-2 text-neutral-500 dark:text-neutral-400' onClick={onBack}>
        <ArrowLeft className='size-3.5' />
        {t('create.back')}
      </Button>

      <div className='flex flex-wrap items-center gap-2 pb-3'>
        <h1 className={tokens.header.title}>{t('create.title')}</h1>
        <Pill tone='violet' mono>
          {LDAP_PROVIDER_TYPE}
        </Pill>
        <button
          type='button'
          onClick={onChangeKind}
          className='cursor-pointer text-xs text-neutral-500 dark:text-neutral-400 underline-offset-2 hover:text-fk-primary-text hover:underline'
        >
          {t('create.change_type')}
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
        title={t('create.save.title')}
        description={t('create.save.description')}
        onCancel={onBack}
        actions={[{ label: t('create.save.action'), onClick: onSubmit }]}
      />
    </PageShell>
  )
}
