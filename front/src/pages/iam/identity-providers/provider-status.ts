import { isProviderIconKey } from '@/components/provider-icon'
import {
  IDENTITY_PROVIDER_NAMESPACE,
  type ProviderTemplate,
} from '@/constants/identity-provider-templates'
import { translate } from '@/lib/i18n'
import { Schemas } from '@/api/api.client'

import IdentityProvider = Schemas.IdentityProviderResponse

export type ProviderProtocol = 'oidc' | 'oauth2' | 'saml' | 'ldap'

export type ProviderHealth = 'healthy' | 'degraded' | 'error'

export interface ProviderStatus {
  health: ProviderHealth
  label: string
  detail: string
}

export const PROVIDER_TYPES = ['oidc', 'oauth2', 'saml', 'ldap'] as const

export const providerTypeLabelKey = (providerId: string) =>
  `provider_type.${providerId.toLowerCase()}`

export const providerTypeLabel = (providerId: string) => {
  const key = providerId.toLowerCase()
  return PROVIDER_TYPES.some((type) => type === key)
    ? translate(`${IDENTITY_PROVIDER_NAMESPACE}:${providerTypeLabelKey(key)}`)
    : providerId
}

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
      label: translate(`${IDENTITY_PROVIDER_NAMESPACE}:status.incomplete.label`),
      detail: translate(`${IDENTITY_PROVIDER_NAMESPACE}:status.incomplete.detail`, {
        count: missing.length,
        keys: missing.join(', '),
      }),
    }
  }

  if (!config.scopes) {
    return {
      health: 'degraded',
      label: translate(`${IDENTITY_PROVIDER_NAMESPACE}:status.no_scope.label`),
      detail: translate(`${IDENTITY_PROVIDER_NAMESPACE}:status.no_scope.detail`),
    }
  }

  return {
    health: 'healthy',
    label: translate(`${IDENTITY_PROVIDER_NAMESPACE}:status.complete.label`),
    detail: translate(`${IDENTITY_PROVIDER_NAMESPACE}:status.complete.detail`),
  }
}

export const providerName = (provider: IdentityProvider) =>
  provider.display_name || provider.alias
