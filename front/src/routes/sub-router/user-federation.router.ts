import { REALM_URL } from '../router'

export const USER_FEDERATION_OVERVIEW_URL = (realmName = ':realmName') =>
  `${REALM_URL(realmName)}/user-federation/overview`
