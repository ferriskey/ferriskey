import { REALM_URL } from '../router'

export const IDENTITY_PROVIDERS_OVERVIEW_URL = (realmName = ':realmName') =>
  `${REALM_URL(realmName)}/identity-providers/overview`
