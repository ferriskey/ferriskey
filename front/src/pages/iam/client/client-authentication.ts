import type { Schemas } from '@/api/api.client'

export type ConfidentialAuthMethod = Exclude<Schemas.TokenEndpointAuthMethod, 'none'>
export type KeySource = 'jwks_uri' | 'jwks'

export const CONFIDENTIAL_AUTH_METHODS: readonly ConfidentialAuthMethod[] = [
  'client_secret_basic',
  'client_secret_post',
  'client_secret_jwt',
  'private_key_jwt',
]

export interface ClientAuthenticationDraft {
  method: ConfidentialAuthMethod
  keySource: KeySource
  jwksUri: string
  jwks: string
}
