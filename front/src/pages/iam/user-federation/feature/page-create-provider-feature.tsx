import { useState } from 'react'
import { Navigate, useNavigate, useParams, useSearchParams } from 'react-router'
import { toast } from 'sonner'
import { useTranslation } from 'react-i18next'
import { useCreateUserFederation } from '@/api/user-federation.api'
import { RouterParams } from '@/routes/router'
import { createLdapProviderSchema } from '@/pages/iam/user-federation/schemas/ldap-provider.schema'
import { USER_FEDERATION_URL } from '@/routes/router'
import PageCreateProvider from '../ui/page-create-provider'
import type { ProviderErrors } from '../ui/provider-form-fields'
import {
  LDAP_PROVIDER_TYPE,
  PRIORITY_SCORE,
  USER_FEDERATION_NAMESPACE,
  buildLdapConfig,
  encodeSecret,
  parseLdapEndpoint,
  type LdapSettings,
  type ProviderPriority,
  type SyncMode,
} from '../provider-config'
import { apiErrorMessage } from '@/lib/api-error'

export default function PageCreateProviderFeature() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const { t } = useTranslation(USER_FEDERATION_NAMESPACE)
  const realm = realm_name ?? 'master'
  const base = USER_FEDERATION_URL(realm)

  const [params] = useSearchParams()
  const kind = params.get('kind')

  const { mutateAsync: createProvider } = useCreateUserFederation()

  const [name, setName] = useState('')
  const [enabled, setEnabled] = useState(true)
  const [priority, setPriority] = useState<ProviderPriority>('Secondary')
  const [ldap, setLdap] = useState<LdapSettings>({
    connectionUrl: '',
    baseDn: '',
    bindDn: '',
    userSearchFilter: '(objectClass=person)',
    useTls: false,
  })
  const [bindPassword, setBindPassword] = useState('')
  const [syncEnabled, setSyncEnabled] = useState(true)
  const [syncMode, setSyncMode] = useState<SyncMode>('Import')
  const [syncIntervalSeconds, setSyncIntervalSeconds] = useState(3600)

  const parsed = createLdapProviderSchema.safeParse({
    type: 'LDAP',
    name,
    enabled,
    priority,
    connectionUrl: ldap.connectionUrl,
    baseDn: ldap.baseDn,
    bindDn: ldap.bindDn,
    bindPassword,
    userSearchFilter: ldap.userSearchFilter,
    syncInterval: syncIntervalSeconds,
    useTls: ldap.useTls,
  })

  const endpoint = parseLdapEndpoint(ldap.connectionUrl, ldap.useTls)

  const issue = (path: string) =>
    parsed.success ? undefined : parsed.error.issues.find((i) => i.path[0] === path)?.message

  const errors: ProviderErrors = {
    name: issue('name'),
    connectionUrl:
      issue('connectionUrl') ??
      (ldap.connectionUrl && !endpoint
        ? t('validation.connection_url_invalid')
        : undefined),
    baseDn: issue('baseDn'),
    userSearchFilter: issue('userSearchFilter'),
    syncInterval: issue('syncInterval'),
  }

  const canSubmit = parsed.success && endpoint !== null

  if (kind !== LDAP_PROVIDER_TYPE) return <Navigate to={`${base}?create=1`} replace />

  const handleSubmit = async () => {
    if (!parsed.success || !endpoint) return

    try {
      await createProvider({
        path: { realm_name: realm },
        body: {
          name,
          enabled,
          provider_type: LDAP_PROVIDER_TYPE,
          priority: PRIORITY_SCORE[priority],
          sync_enabled: syncEnabled,
          sync_mode: syncMode,
          sync_interval_minutes: Math.floor(syncIntervalSeconds / 60),
          config: buildLdapConfig(ldap, endpoint, encodeSecret(bindPassword)),
        },
      })
      toast.success(t('create.toast.created'))
      navigate(base)
    } catch (error) {
      toast.error(apiErrorMessage(error, t('create.toast.failed')))
    }
  }

  return (
    <PageCreateProvider
      name={name}
      enabled={enabled}
      priority={priority}
      ldap={ldap}
      bindPassword={bindPassword}
      syncEnabled={syncEnabled}
      syncMode={syncMode}
      syncIntervalSeconds={syncIntervalSeconds}
      errors={errors}
      canSubmit={canSubmit}
      onNameChange={setName}
      onEnabledChange={setEnabled}
      onPriorityChange={setPriority}
      onLdapChange={(patch) => setLdap((s) => ({ ...s, ...patch }))}
      onBindPasswordChange={setBindPassword}
      onSyncEnabledChange={setSyncEnabled}
      onSyncModeChange={setSyncMode}
      onSyncIntervalChange={setSyncIntervalSeconds}
      onBack={() => navigate(base)}
      onChangeKind={() => navigate(`${base}?create=1`)}
      onSubmit={handleSubmit}
    />
  )
}
