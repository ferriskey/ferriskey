import { REALM_URL } from '../router'

export const USER_MANAGEMENT_URL = (realmName = ':realmName') =>
  `${REALM_URL(realmName)}/console/user-management`

export const IDENTITIES_URL = (realmName = ':realmName') =>
  `${USER_MANAGEMENT_URL(realmName)}/identities`

export const UM_ORGANIZATIONS_URL = (realmName = ':realmName') =>
  `${USER_MANAGEMENT_URL(realmName)}/organizations`

export const UM_ORGANIZATION_CREATE_URL = (realmName = ':realmName') =>
  `${UM_ORGANIZATIONS_URL(realmName)}/create`

export const UM_ROLES_URL = (realmName = ':realmName') =>
  `${USER_MANAGEMENT_URL(realmName)}/roles`

export const UM_ROLE_CREATE_URL = (realmName = ':realmName') =>
  `${UM_ROLES_URL(realmName)}/create`
