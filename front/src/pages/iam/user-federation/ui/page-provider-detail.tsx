import { ArrowLeft, Database, KeyRound, Plug, RefreshCw, Server } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/kit/button'
import SaveBar from '@/components/kit/save-bar'
import { DetailHeader, IconTile, PageShell, PageTabs, Pill, StatusDot, type TabItem } from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import { Schemas } from '@/api/api.client'
import {
  LDAP_PROVIDER_TYPE,
  PRIORITY_LABEL_KEY,
  USER_FEDERATION_NAMESPACE,
  formatDuration,
  isLdapLike,
  priorityFromScore,
  type LdapSettings,
  type ProviderPriority,
  type SyncMode,
} from '../provider-config'
import ProviderSettingsTab from './provider-settings-tab'
import ProviderSyncTab from './provider-sync-tab'
import type { ProviderErrors } from './provider-form-fields'

import ProviderResponse = Schemas.ProviderResponse
import SyncUsersResponse = Schemas.SyncUsersResponse

export interface ConnectionTestResult {
  success: boolean
  message: string
  latencyMs: number | null
}

export interface PageProviderDetailProps {
  provider?: ProviderResponse
  isLoading: boolean
  tab: string
  tabs: TabItem[]
  name: string
  enabled: boolean
  priority: ProviderPriority
  ldap: LdapSettings
  bindPassword: string
  secretStored: boolean
  secretRequired: boolean
  syncEnabled: boolean
  syncMode: SyncMode
  syncIntervalSeconds: number
  errors: ProviderErrors
  dirtyCount: number
  canSave: boolean
  isTesting: boolean
  isSyncing: boolean
  testResult?: ConnectionTestResult
  lastRun?: SyncUsersResponse
  onNameChange: (v: string) => void
  onEnabledChange: (v: boolean) => void
  onPriorityChange: (v: ProviderPriority) => void
  onLdapChange: (patch: Partial<LdapSettings>) => void
  onBindPasswordChange: (v: string) => void
  onSyncEnabledChange: (v: boolean) => void
  onSyncModeChange: (v: SyncMode) => void
  onSyncIntervalChange: (seconds: number) => void
  onBack: () => void
  onTestConnection: () => void
  onSyncUsers: () => void
  onDiscard: () => void
  onSave: () => void
  onDelete: () => void
}

const KERBEROS_PROVIDER_TYPE = 'Kerberos'

const providerIcon = (providerType: string) =>
  isLdapLike(providerType) ? (
    <Database className='size-6' strokeWidth={1.5} />
  ) : providerType === KERBEROS_PROVIDER_TYPE ? (
    <KeyRound className='size-6' strokeWidth={1.5} />
  ) : (
    <Server className='size-6' strokeWidth={1.5} />
  )

export default function PageProviderDetail({
  provider,
  isLoading,
  tab,
  tabs,
  name,
  enabled,
  priority,
  ldap,
  bindPassword,
  secretStored,
  secretRequired,
  syncEnabled,
  syncMode,
  syncIntervalSeconds,
  errors,
  dirtyCount,
  canSave,
  isTesting,
  isSyncing,
  testResult,
  lastRun,
  onNameChange,
  onEnabledChange,
  onPriorityChange,
  onLdapChange,
  onBindPasswordChange,
  onSyncEnabledChange,
  onSyncModeChange,
  onSyncIntervalChange,
  onBack,
  onTestConnection,
  onSyncUsers,
  onDiscard,
  onSave,
  onDelete,
}: PageProviderDetailProps) {
  const { t } = useTranslation(USER_FEDERATION_NAMESPACE)

  if (isLoading) {
    return (
      <PageShell>
        <div className='h-4 w-32 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
        <div className='mt-4 flex items-center gap-3'>
          <div className='size-15 animate-pulse rounded-md bg-neutral-100 dark:bg-fk-raised' />
          <div className='space-y-2'>
            <div className='h-5 w-48 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
            <div className='h-4 w-32 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
          </div>
        </div>
      </PageShell>
    )
  }

  if (!provider) {
    return (
      <PageShell>
        <Button variant='ghost' size='sm' className='-ml-2 mb-2 text-neutral-500 dark:text-neutral-400' onClick={onBack}>
          <ArrowLeft className='size-3.5' />
          {t('detail.back')}
        </Button>
        <div className={cn(tokens.surface.panel, 'grid place-items-center px-6 py-16')}>
          <p className='text-sm font-medium text-neutral-700 dark:text-neutral-300'>
            {t('detail.not_found.title')}
          </p>
          <p className='mt-1 max-w-sm text-center text-sm text-neutral-500 dark:text-neutral-400'>
            {t('detail.not_found.description')}
          </p>
        </div>
      </PageShell>
    )
  }

  const testable = isLdapLike(provider.provider_type)

  return (
    <PageShell>
      <DetailHeader
        onBack={onBack}
        backLabel={t('detail.back')}
        icon={
          <IconTile
            tone={provider.provider_type === LDAP_PROVIDER_TYPE ? 'violet' : 'info'}
            className='size-15'
          >
            {providerIcon(provider.provider_type)}
          </IconTile>
        }
        title={name}
        pills={
          <>
            <Pill tone={provider.provider_type === LDAP_PROVIDER_TYPE ? 'violet' : 'info'} mono>
              {provider.provider_type}
            </Pill>
            <Pill mono>{provider.sync_mode}</Pill>
            <Pill>{t(PRIORITY_LABEL_KEY[priorityFromScore(provider.priority)])}</Pill>
            <Pill tone={enabled ? 'success' : 'neutral'}>
              <StatusDot on={enabled} />
              {enabled ? t('detail.status.enabled') : t('detail.status.disabled')}
            </Pill>
          </>
        }
        meta={
          <div className='flex shrink-0 flex-wrap gap-2'>
            <Button
              variant='outline'
              onClick={onTestConnection}
              disabled={!testable || isTesting || isSyncing}
              title={testable ? undefined : t('detail.actions.test_unavailable')}
            >
              <Plug />
              {isTesting ? t('detail.actions.testing') : t('detail.actions.test')}
            </Button>
            <Button
              variant='outline'
              onClick={onSyncUsers}
              disabled={!testable || !provider.enabled || isTesting || isSyncing}
              title={
                !testable
                  ? t('detail.actions.sync_unavailable')
                  : !provider.enabled
                    ? t('detail.actions.sync_disabled')
                    : undefined
              }
            >
              <RefreshCw className={cn(isSyncing && 'animate-spin')} />
              {isSyncing ? t('detail.actions.syncing') : t('detail.actions.sync')}
            </Button>
          </div>
        }
      />

      {testResult && (
        <div
          className={cn(
            'mt-4 flex flex-wrap items-center gap-2 rounded-sm border px-3 py-2 text-[13px]',
            testResult.success
              ? 'border-fk-success-border bg-fk-success-soft/50'
              : 'border-fk-danger-border bg-fk-danger-soft/40'
          )}
        >
          <Pill tone={testResult.success ? 'success' : 'danger'} mono>
            {testResult.success ? t('detail.test.reachable') : t('detail.test.unreachable')}
          </Pill>
          <span className='text-neutral-700 dark:text-neutral-300'>{testResult.message}</span>
          {testResult.latencyMs !== null && (
            <span className='tnum text-xs text-neutral-500 dark:text-neutral-400'>
              {formatDuration(testResult.latencyMs)}
            </span>
          )}
        </div>
      )}

      {!provider.enabled && (
        <div className='mt-4 rounded-sm border border-fk-line bg-neutral-50 px-3 py-2 text-xs text-neutral-600 dark:bg-fk-surface dark:text-neutral-400'>
          {t('detail.disabled_notice')}
        </div>
      )}

      <PageTabs tabs={tabs} value={tab} className='mt-5'>
        <div className={tokens.page.blockGap}>
          {tab === 'settings' && (
            <ProviderSettingsTab
              provider={provider}
              name={name}
              enabled={enabled}
              priority={priority}
              ldap={ldap}
              bindPassword={bindPassword}
              secretStored={secretStored}
              secretRequired={secretRequired}
              errors={errors}
              onNameChange={onNameChange}
              onEnabledChange={onEnabledChange}
              onPriorityChange={onPriorityChange}
              onLdapChange={onLdapChange}
              onBindPasswordChange={onBindPasswordChange}
              onDelete={onDelete}
            />
          )}

          {tab === 'sync' && (
            <ProviderSyncTab
              provider={provider}
              syncEnabled={syncEnabled}
              syncMode={syncMode}
              syncIntervalSeconds={syncIntervalSeconds}
              errors={errors}
              lastRun={lastRun}
              onSyncEnabledChange={onSyncEnabledChange}
              onSyncModeChange={onSyncModeChange}
              onSyncIntervalChange={onSyncIntervalChange}
            />
          )}
        </div>
      </PageTabs>

      <SaveBar
        show={dirtyCount > 0}
        title={t('detail.save.title', { count: dirtyCount })}
        description={canSave ? t('detail.save.description') : t('detail.save.blocked')}
        onCancel={onDiscard}
        cancelLabel={t('detail.save.cancel')}
        actions={canSave ? [{ label: t('detail.save.action'), onClick: onSave }] : []}
      />
    </PageShell>
  )
}
