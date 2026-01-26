import { REALM_URL } from '../router'

export const IDENTITY_PROVIDERS_URL = (realmName = ':realmName') =>
  `${REALM_URL(realmName)}/identity-providers`

export const IDENTITY_PROVIDER_URL = (
  realmName = ':realmName',
  providerId = ':providerId'
) => `${IDENTITY_PROVIDERS_URL(realmName)}/${providerId}`

export const IDENTITY_PROVIDER_OVERVIEW_URL = '/overview'
export const IDENTITY_PROVIDER_CREATE_URL = '/create'

export type IdentityProviderRouterParams = {
  realm_name: string
  providerId: string
}
