export const PKCE_CONFIG_KEY = 'use_pkce'

export type ProviderDraft = {
  displayName: string
  enabled: boolean
  usePkce: boolean
}

export type ProviderUpdateBody = {
  display_name: string
  enabled: boolean
  use_pkce?: boolean
}

export const storedUsePkce = (config: unknown): boolean =>
  typeof config === 'object' && config !== null
    ? (config as Record<string, unknown>)[PKCE_CONFIG_KEY] === true
    : false

export const countProviderChanges = (draft: ProviderDraft, stored: ProviderDraft): number =>
  (draft.displayName !== stored.displayName ? 1 : 0) +
  (draft.enabled !== stored.enabled ? 1 : 0) +
  (draft.usePkce !== stored.usePkce ? 1 : 0)

export const buildProviderUpdateBody = (
  draft: ProviderDraft,
  stored: ProviderDraft
): ProviderUpdateBody => {
  const body: ProviderUpdateBody = {
    display_name: draft.displayName,
    enabled: draft.enabled,
  }

  if (draft.usePkce !== stored.usePkce) {
    body.use_pkce = draft.usePkce
  }

  return body
}
