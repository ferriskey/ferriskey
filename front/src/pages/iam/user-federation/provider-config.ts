import { formatDateTime } from '@/shared/format-date'

export type ProviderPriority = 'Primary' | 'Secondary' | 'Development' | 'Legacy'
export type SyncMode = 'Import' | 'Force' | 'LinkOnly'

export const MASKED_SECRET = '********'

export const PRIORITY_SCORE: Record<ProviderPriority, number> = {
  Primary: 0,
  Secondary: 10,
  Development: 20,
  Legacy: 30,
}

export const PRIORITY_ORDER: ProviderPriority[] = [
  'Primary',
  'Secondary',
  'Development',
  'Legacy',
]

export const priorityFromScore = (score: number): ProviderPriority => {
  if (score === 0) return 'Primary'
  if (score === 10) return 'Secondary'
  if (score === 20) return 'Development'
  return 'Legacy'
}

export const SYNC_MODES: SyncMode[] = ['Import', 'Force', 'LinkOnly']

export const asSyncMode = (value: string): SyncMode =>
  (SYNC_MODES as string[]).includes(value) ? (value as SyncMode) : 'Import'

export const isLdapLike = (providerType: string) =>
  providerType === 'Ldap' || providerType === 'ActiveDirectory'

export interface LdapConfig {
  connection?: {
    server_url?: string
    port?: number
    use_tls?: boolean
    use_starttls?: boolean
    connection_timeout_seconds?: number
  }
  bind?: {
    bind_dn?: string
    bind_password_encrypted?: string
  }
  search?: {
    base_dn?: string
    user_search_filter?: string
  }
  attributes?: Record<string, string>
}

export interface LdapSettings {
  connectionUrl: string
  baseDn: string
  bindDn: string
  userSearchFilter: string
  useTls: boolean
}

export interface LdapEndpoint {
  host: string
  port: number
}

export const asLdapConfig = (config: unknown): LdapConfig =>
  config && typeof config === 'object' ? (config as LdapConfig) : {}

export const ldapConnectionUrl = (config: LdapConfig) => {
  const server = config.connection?.server_url ?? ''
  if (!server) return ''
  const useTls = config.connection?.use_tls ?? false
  const port = config.connection?.port ?? (useTls ? 636 : 389)
  return `${useTls ? 'ldaps' : 'ldap'}://${server}:${port}`
}

export const ldapSettingsFrom = (config: LdapConfig): LdapSettings => ({
  connectionUrl: ldapConnectionUrl(config),
  baseDn: config.search?.base_dn ?? '',
  bindDn: config.bind?.bind_dn ?? '',
  userSearchFilter: config.search?.user_search_filter ?? '(objectClass=person)',
  useTls: config.connection?.use_tls ?? false,
})

export const storedSecret = (config: LdapConfig) =>
  config.bind?.bind_password_encrypted ?? ''

export const secretIsMasked = (config: LdapConfig) =>
  storedSecret(config) === MASKED_SECRET

export const parseLdapEndpoint = (raw: string, useTls: boolean): LdapEndpoint | null => {
  const trimmed = raw.trim()
  if (!trimmed) return null
  try {
    const url = new URL(trimmed.startsWith('ldap') ? trimmed : `ldap://${trimmed}`)
    if (!url.hostname) return null
    const port = url.port ? Number(url.port) : useTls ? 636 : 389
    if (!Number.isFinite(port) || port <= 0) return null
    return { host: url.hostname, port }
  } catch {
    return null
  }
}

export const buildLdapConfig = (
  settings: LdapSettings,
  endpoint: LdapEndpoint,
  bindPasswordEncrypted: string,
  previous: LdapConfig = {}
): LdapConfig => ({
  connection: {
    server_url: endpoint.host,
    port: endpoint.port,
    use_tls: settings.useTls,
    use_starttls: previous.connection?.use_starttls ?? false,
    connection_timeout_seconds: previous.connection?.connection_timeout_seconds ?? 30,
  },
  bind: {
    bind_dn: settings.bindDn,
    bind_password_encrypted: bindPasswordEncrypted,
  },
  search: {
    base_dn: settings.baseDn,
    user_search_filter: settings.userSearchFilter,
  },
  attributes: previous.attributes ?? {
    username: 'uid',
    email: 'mail',
    first_name: 'givenName',
    last_name: 'sn',
  },
})

export const encodeSecret = (value: string) => btoa(value)

export const formatDuration = (ms: number | null) => {
  if (ms === null) return '—'
  return ms < 1000 ? `${ms} ms` : `${(ms / 1000).toFixed(1)} s`
}

export const formatSyncedAt = (iso?: string | null) =>
  iso ? formatDateTime(iso) : null

export { formatDateTime }

export const humanizeConfigKey = (key: string) =>
  key.charAt(0).toUpperCase() + key.slice(1).replace(/_/g, ' ')

export const isSecretKey = (key: string) =>
  key.includes('password') || key.includes('secret') || key.includes('encrypted')

export const flattenConfig = (config: unknown, prefix = ''): [string, string][] => {
  if (!config || typeof config !== 'object') return []
  return Object.entries(config as Record<string, unknown>).flatMap(([key, value]) => {
    const path = prefix ? `${prefix}.${key}` : key
    if (value && typeof value === 'object' && !Array.isArray(value)) {
      return flattenConfig(value, path)
    }
    return [[path, value === null || value === undefined ? '—' : String(value)]] as [
      string,
      string,
    ][]
  })
}
