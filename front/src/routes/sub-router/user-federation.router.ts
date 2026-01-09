import { REALM_URL } from '../router'

export const USER_FEDERATION_URL = (realmName = ':realmName') =>
  `${REALM_URL(realmName)}/user-federation`

export const USER_FEDERATION_OVERVIEW_URL = (realmName = ':realmName') =>
  `${USER_FEDERATION_URL(realmName)}/overview`

export const USER_FEDERATION_CREATE_URL = '/create'

export const USER_FEDERATION_LDAP_DETAIL_URL = (realmName: string, id: string) =>
  `${USER_FEDERATION_URL(realmName)}/ldap/${id}`

export const USER_FEDERATION_KERBEROS_DETAIL_URL = (realmName: string, id: string) =>
  `${USER_FEDERATION_URL(realmName)}/kerberos/${id}`
