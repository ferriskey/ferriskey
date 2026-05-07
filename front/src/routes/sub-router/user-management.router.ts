import { REALM_URL } from '../router'

export const USER_MANAGEMENT_URL = (realmName = ':realmName') =>
  `${REALM_URL(realmName)}/user-management`

export const IDENTITIES_URL = (realmName = ':realmName') =>
  `${USER_MANAGEMENT_URL(realmName)}/identities`
