export const PKCE_CONFIG_KEY = 'use_pkce'

export const FORM_CONFIG_KEYS = [
  'client_id',
  'client_secret',
  'authorization_url',
  'token_url',
  'userinfo_url',
  'scopes',
  PKCE_CONFIG_KEY,
] as const

export type ProviderDraft = {
  displayName: string
  enabled: boolean
  clientId: string
  clientSecret: string
  authorizationUrl: string
  tokenUrl: string
  userinfoUrl: string
  scopes: string[]
  usePkce: boolean
}

export type ProviderUpdateBody = {
  display_name: string
  enabled: boolean
  client_id?: string
  client_secret?: string
  authorization_url?: string
  token_url?: string
  userinfo_url?: string
  scopes?: string
  use_pkce?: boolean
}

export type ProviderDraftErrors = Partial<
  Record<'clientId' | 'authorizationUrl' | 'tokenUrl' | 'userinfoUrl', string>
>

const asRecord = (config: unknown): Record<string, unknown> =>
  typeof config === 'object' && config !== null ? (config as Record<string, unknown>) : {}

const configString = (config: unknown, key: string): string => {
  const value = asRecord(config)[key]
  return typeof value === 'string' ? value : ''
}

export const storedUsePkce = (config: unknown): boolean =>
  asRecord(config)[PKCE_CONFIG_KEY] === true

export const extraConfigEntries = (config: unknown): [string, unknown][] =>
  Object.entries(asRecord(config)).filter(
    ([key]) => !FORM_CONFIG_KEYS.some((known) => known === key)
  )

export const storedProviderDraft = (provider: {
  display_name?: string | null
  enabled: boolean
  config?: unknown
}): ProviderDraft => ({
  displayName: provider.display_name ?? '',
  enabled: provider.enabled,
  clientId: configString(provider.config, 'client_id'),
  clientSecret: '',
  authorizationUrl: configString(provider.config, 'authorization_url'),
  tokenUrl: configString(provider.config, 'token_url'),
  userinfoUrl: configString(provider.config, 'userinfo_url'),
  scopes: configString(provider.config, 'scopes').split(/\s+/).filter(Boolean),
  usePkce: storedUsePkce(provider.config),
})

const isUrl = (value: string): boolean => {
  try {
    new URL(value)
    return true
  } catch {
    return false
  }
}

export const validateProviderDraft = (draft: ProviderDraft): ProviderDraftErrors => {
  const errors: ProviderDraftErrors = {}

  if (!draft.clientId) errors.clientId = 'validation.client_id_required'

  if (!draft.authorizationUrl) errors.authorizationUrl = 'validation.url_required'
  else if (!isUrl(draft.authorizationUrl)) errors.authorizationUrl = 'validation.url_invalid'

  if (!draft.tokenUrl) errors.tokenUrl = 'validation.url_required'
  else if (!isUrl(draft.tokenUrl)) errors.tokenUrl = 'validation.url_invalid'

  if (draft.userinfoUrl && !isUrl(draft.userinfoUrl)) errors.userinfoUrl = 'validation.url_invalid'

  return errors
}

const sameScopes = (left: string[], right: string[]): boolean =>
  left.length === right.length && left.every((scope, index) => scope === right[index])

export const countProviderChanges = (draft: ProviderDraft, stored: ProviderDraft): number =>
  (draft.displayName !== stored.displayName ? 1 : 0) +
  (draft.enabled !== stored.enabled ? 1 : 0) +
  (draft.clientId !== stored.clientId ? 1 : 0) +
  (draft.clientSecret !== '' ? 1 : 0) +
  (draft.authorizationUrl !== stored.authorizationUrl ? 1 : 0) +
  (draft.tokenUrl !== stored.tokenUrl ? 1 : 0) +
  (draft.userinfoUrl !== stored.userinfoUrl ? 1 : 0) +
  (sameScopes(draft.scopes, stored.scopes) ? 0 : 1) +
  (draft.usePkce !== stored.usePkce ? 1 : 0)

export const buildProviderUpdateBody = (
  draft: ProviderDraft,
  stored: ProviderDraft
): ProviderUpdateBody => {
  const body: ProviderUpdateBody = {
    display_name: draft.displayName,
    enabled: draft.enabled,
  }

  if (draft.clientId !== stored.clientId) body.client_id = draft.clientId
  if (draft.clientSecret !== '') body.client_secret = draft.clientSecret
  if (draft.authorizationUrl !== stored.authorizationUrl) {
    body.authorization_url = draft.authorizationUrl
  }
  if (draft.tokenUrl !== stored.tokenUrl) body.token_url = draft.tokenUrl
  if (draft.userinfoUrl !== stored.userinfoUrl) body.userinfo_url = draft.userinfoUrl
  if (!sameScopes(draft.scopes, stored.scopes)) body.scopes = draft.scopes.join(' ')
  if (draft.usePkce !== stored.usePkce) body.use_pkce = draft.usePkce

  return body
}
