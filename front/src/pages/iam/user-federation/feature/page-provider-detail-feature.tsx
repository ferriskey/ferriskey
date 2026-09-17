import { useMemo, useState } from 'react'
import { useNavigate, useParams } from 'react-router'
import { toast } from 'sonner'
import { useTranslation } from 'react-i18next'
import {
  useDeleteUserFederation,
  useGetUserFederation,
  useSyncUsers,
  useTestUserFederationConnection,
  useUpdateUserFederation,
} from '@/api/user-federation.api'
import { useRouteTabs } from '@/components/kit'
import { useConfettiFireworks } from '@/hooks/use-confetti-fireworks'
import { RouterParams } from '@/routes/router'
import { createLdapProviderSchema } from '@/pages/iam/user-federation/schemas/ldap-provider.schema'
import { USER_FEDERATION_URL } from '@/routes/router'
import { Schemas } from '@/api/api.client'
import PageProviderDetail, { type ConnectionTestResult } from '../ui/page-provider-detail'
import type { ProviderErrors } from '../ui/provider-form-fields'
import {
  PRIORITY_SCORE,
  USER_FEDERATION_NAMESPACE,
  asLdapConfig,
  asSyncMode,
  buildLdapConfig,
  encodeSecret,
  isLdapLike,
  ldapSettingsFrom,
  parseLdapEndpoint,
  priorityFromScore,
  secretIsMasked,
  storedSecret,
  type LdapSettings,
  type ProviderPriority,
  type SyncMode,
} from '../provider-config'

import SyncUsersResponse = Schemas.SyncUsersResponse
import { useCrumbLabel } from '@/components/shell/crumb-store'

interface Draft {
  key: string
  name: string
  enabled: boolean
  priority: ProviderPriority
  ldap: LdapSettings
  bindPassword: string
  syncEnabled: boolean
  syncMode: SyncMode
  syncIntervalSeconds: number
}

const EMPTY_DRAFT: Draft = {
  key: '',
  name: '',
  enabled: true,
  priority: 'Secondary',
  ldap: {
    connectionUrl: '',
    baseDn: '',
    bindDn: '',
    userSearchFilter: '(objectClass=person)',
    useTls: false,
  },
  bindPassword: '',
  syncEnabled: true,
  syncMode: 'Import',
  syncIntervalSeconds: 3600,
}

const PROVIDER_TABS = [
  { key: 'settings', labelKey: 'detail.tabs.settings' },
  { key: 'sync', labelKey: 'detail.tabs.sync' },
] as const

const latencyOf = (details: unknown): number | null => {
  if (!details || typeof details !== 'object') return null
  const value = (details as Record<string, unknown>).latency_ms
  return typeof value === 'number' ? value : null
}

export default function PageProviderDetailFeature() {
  const { realm_name, provider_id } = useParams<RouterParams & { provider_id: string }>()
  const navigate = useNavigate()
  const { t } = useTranslation(USER_FEDERATION_NAMESPACE)
  const realm = realm_name ?? 'master'
  const base = USER_FEDERATION_URL(realm)
  const providerId = provider_id ?? ''

  const { data: provider, isLoading } = useGetUserFederation(realm, providerId)
  const { mutateAsync: updateProvider } = useUpdateUserFederation()
  const { mutateAsync: deleteProvider } = useDeleteUserFederation()
  const { mutateAsync: testConnection, isPending: isTesting } =
    useTestUserFederationConnection()
  const { mutateAsync: syncUsers, isPending: isSyncing } = useSyncUsers()
  const { fire } = useConfettiFireworks()

  const translatedTabs = useMemo(
    () => PROVIDER_TABS.map((item) => ({ key: item.key, label: t(item.labelKey) })),
    [t]
  )

  const { value: tab, tabs } = useRouteTabs(`${base}/${providerId}`, translatedTabs)

  const [draft, setDraft] = useState<Draft>(EMPTY_DRAFT)
  const [testResult, setTestResult] = useState<ConnectionTestResult>()
  const [lastRun, setLastRun] = useState<SyncUsersResponse>()

  const config = asLdapConfig(provider?.config)

  const pristine: Draft = provider
    ? {
        key: provider.id,
        name: provider.name,
        enabled: provider.enabled,
        priority: priorityFromScore(provider.priority),
        ldap: ldapSettingsFrom(config),
        bindPassword: '',
        syncEnabled: provider.sync_enabled,
        syncMode: asSyncMode(provider.sync_mode),
        syncIntervalSeconds: (provider.sync_interval_minutes ?? 60) * 60,
      }
    : EMPTY_DRAFT

  if (provider && draft.key !== provider.id) setDraft(pristine)

  const current = draft.key === pristine.key ? draft : pristine

  const patch = (next: Partial<Draft>) => setDraft((d) => ({ ...d, ...next }))
  const patchLdap = (next: Partial<LdapSettings>) =>
    setDraft((d) => ({ ...d, ldap: { ...d.ldap, ...next } }))

  const parsed = createLdapProviderSchema.safeParse({
    type: 'LDAP',
    name: current.name,
    enabled: current.enabled,
    priority: current.priority,
    connectionUrl: current.ldap.connectionUrl,
    baseDn: current.ldap.baseDn,
    bindDn: current.ldap.bindDn,
    bindPassword: current.bindPassword,
    userSearchFilter: current.ldap.userSearchFilter,
    syncInterval: current.syncIntervalSeconds,
    useTls: current.ldap.useTls,
  })

  const endpoint = parseLdapEndpoint(current.ldap.connectionUrl, current.ldap.useTls)
  const editable = provider ? isLdapLike(provider.provider_type) : false

  const issue = (path: string) =>
    parsed.success ? undefined : parsed.error.issues.find((i) => i.path[0] === path)?.message

  const definitionDirty =
    current.name !== pristine.name ||
    current.enabled !== pristine.enabled ||
    current.priority !== pristine.priority

  const configDirty =
    editable &&
    (current.ldap.connectionUrl !== pristine.ldap.connectionUrl ||
      current.ldap.baseDn !== pristine.ldap.baseDn ||
      current.ldap.bindDn !== pristine.ldap.bindDn ||
      current.ldap.userSearchFilter !== pristine.ldap.userSearchFilter ||
      current.ldap.useTls !== pristine.ldap.useTls ||
      current.bindPassword.length > 0)

  const syncDirty =
    current.syncEnabled !== pristine.syncEnabled ||
    current.syncMode !== pristine.syncMode ||
    current.syncIntervalSeconds !== pristine.syncIntervalSeconds

  const secretRequired =
    configDirty && secretIsMasked(config) && current.bindPassword.length === 0

  const errors: ProviderErrors = editable
    ? {
        name: issue('name'),
        connectionUrl:
          issue('connectionUrl') ??
          (current.ldap.connectionUrl && !endpoint
            ? t('validation.connection_url_invalid')
            : undefined),
        baseDn: issue('baseDn'),
        userSearchFilter: issue('userSearchFilter'),
        syncInterval: issue('syncInterval'),
      }
    : { name: issue('name'), syncInterval: issue('syncInterval') }

  const dirtyCount =
    (definitionDirty ? 1 : 0) + (configDirty ? 1 : 0) + (syncDirty ? 1 : 0)

  const canSave =
    !secretRequired &&
    !errors.name &&
    !errors.syncInterval &&
    (!editable || (parsed.success && endpoint !== null))

  const handleSave = async () => {
    if (!provider || !canSave) return

    const nextSecret = current.bindPassword
      ? encodeSecret(current.bindPassword)
      : storedSecret(config)

    try {
      await updateProvider({
        path: { realm_name: realm, id: provider.id },
        body: {
          name: current.name,
          enabled: current.enabled,
          provider_type: provider.provider_type,
          priority: PRIORITY_SCORE[current.priority],
          sync_enabled: current.syncEnabled,
          sync_mode: current.syncMode,
          sync_interval_minutes: Math.floor(current.syncIntervalSeconds / 60),
          ...(configDirty && endpoint
            ? { config: buildLdapConfig(current.ldap, endpoint, nextSecret, config) }
            : {}),
        },
      })
      setDraft((d) => ({ ...d, key: '', bindPassword: '' }))
      toast.success(t('detail.toast.updated'))
    } catch {
      toast.error(t('detail.toast.update_failed'))
    }
  }

  const handleTestConnection = async () => {
    if (!provider) return

    try {
      const result = await testConnection({
        path: { realm_name: realm, id: provider.id },
      })
      setTestResult({
        success: result.success,
        message: result.message,
        latencyMs: latencyOf(result.details),
      })
      if (result.success) {
        fire()
        toast.success(t('detail.toast.test_success'), { description: result.message })
      } else {
        toast.error(t('detail.toast.test_failed'), { description: result.message })
      }
    } catch {
      toast.error(t('detail.toast.test_error'))
    }
  }

  const handleSyncUsers = async () => {
    if (!provider) return

    try {
      const result = await syncUsers({ path: { realm_name: realm, id: provider.id } })
      setLastRun(result)
      const stats = [
        result.created > 0 ? t('detail.toast.stats.created', { total: result.created }) : null,
        result.updated > 0 ? t('detail.toast.stats.updated', { total: result.updated }) : null,
        result.disabled > 0
          ? t('detail.toast.stats.disabled', { total: result.disabled })
          : null,
        result.failed > 0 ? t('detail.toast.stats.failed', { total: result.failed }) : null,
      ]
        .filter(Boolean)
        .join(', ')
      toast.success(t('detail.toast.sync_completed'), {
        description:
          stats || t('detail.toast.sync_processed', { count: result.total_processed }),
      })
    } catch {
      toast.error(t('detail.toast.sync_failed'))
    }
  }

  const handleDelete = async () => {
    if (!provider) return
    try {
      await deleteProvider({ path: { realm_name: realm, id: provider.id } })
      toast.success(t('detail.toast.deleted'))
      navigate(base)
    } catch {
      toast.error(t('detail.toast.delete_failed'))
    }
  }


  useCrumbLabel(provider_id, provider?.name)

  return (
    <PageProviderDetail
      provider={provider}
      isLoading={isLoading}
      tab={tab}
      tabs={tabs}
      name={current.name}
      enabled={current.enabled}
      priority={current.priority}
      ldap={current.ldap}
      bindPassword={current.bindPassword}
      secretStored={storedSecret(config).length > 0}
      secretRequired={secretRequired}
      syncEnabled={current.syncEnabled}
      syncMode={current.syncMode}
      syncIntervalSeconds={current.syncIntervalSeconds}
      errors={errors}
      dirtyCount={dirtyCount}
      canSave={canSave}
      isTesting={isTesting}
      isSyncing={isSyncing}
      testResult={testResult}
      lastRun={lastRun}
      onNameChange={(v) => patch({ name: v })}
      onEnabledChange={(v) => patch({ enabled: v })}
      onPriorityChange={(v) => patch({ priority: v })}
      onLdapChange={patchLdap}
      onBindPasswordChange={(v) => patch({ bindPassword: v })}
      onSyncEnabledChange={(v) => patch({ syncEnabled: v })}
      onSyncModeChange={(v) => patch({ syncMode: v })}
      onSyncIntervalChange={(v) => patch({ syncIntervalSeconds: v })}
      onBack={() => navigate(base)}
      onTestConnection={handleTestConnection}
      onSyncUsers={handleSyncUsers}
      onDiscard={() => setDraft(pristine)}
      onSave={handleSave}
      onDelete={handleDelete}
    />
  )
}
