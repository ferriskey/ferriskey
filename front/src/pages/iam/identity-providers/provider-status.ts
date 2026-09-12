import { isProviderIconKey } from '@/components/provider-icon'
import type { ProviderTemplate } from '@/constants/identity-provider-templates'
import { Schemas } from '@/api/api.client'

import IdentityProvider = Schemas.IdentityProviderResponse

export type ProviderProtocol = 'oidc' | 'oauth2' | 'saml' | 'ldap'

export type ProviderHealth = 'healthy' | 'degraded' | 'error'

export interface ProviderStatus {
  health: ProviderHealth
  label: string
  detail: string
}

export const PROVIDER_TYPE_LABELS: Record<string, string> = {
  oidc: 'OIDC',
  oauth2: 'OAuth2',
  saml: 'SAML',
  ldap: 'LDAP',
}

export const providerTypeLabel = (providerId: string) =>
  PROVIDER_TYPE_LABELS[providerId.toLowerCase()] ?? providerId

export const providerLogo = (providerId: string): ProviderTemplate['icon'] => {
  const key = providerId.toLowerCase()
  return isProviderIconKey(key) ? key : 'custom'
}

export const providerConfig = (provider: IdentityProvider): Record<string, unknown> =>
  provider.config && typeof provider.config === 'object'
    ? (provider.config as Record<string, unknown>)
    : {}

const REQUIRED_CONFIG_KEYS = [
  'client_id',
  'client_secret',
  'authorization_url',
  'token_url',
] as const

export const isSecretKey = (key: string) =>
  key.includes('secret') || key.includes('credential') || key.includes('certificate')

export const humanizeConfigKey = (key: string) =>
  key
    .split('_')
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(' ')

export const providerStatus = (provider: IdentityProvider): ProviderStatus => {
  const config = providerConfig(provider)
  const missing = REQUIRED_CONFIG_KEYS.filter((key) => !config[key])

  if (missing.length > 0) {
    return {
      health: 'error',
      label: 'incomplete',
      detail: `Broker login will fail — ${missing.join(', ')} missing from the configuration.`,
    }
  }

  if (!config.scopes) {
    return {
      health: 'degraded',
      label: 'no scope',
      detail: 'No scope is requested; most providers reject an authorization request without one.',
    }
  }

  return {
    health: 'healthy',
    label: 'complete',
    detail: 'Every value the broker needs is recorded.',
  }
}

export const providerName = (provider: IdentityProvider) =>
  provider.display_name || provider.alias
