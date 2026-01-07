import { REALM_URL } from '../router'

export const USER_FEDERATION_URL = (realmName = ':realmName') =>
  `${REALM_URL(realmName)}/user-federation`

export const USER_FEDERATION_OVERVIEW_URL = (realmName = ':realmName') =>
  `${USER_FEDERATION_URL(realmName)}/overview`

export const USER_FEDERATION_CREATE_URL = '/create'
