import { CONSOLE_URL } from '@/routes/router'

const USER_MANAGEMENT_URL = (realmName = ':realm_name') =>
  `${CONSOLE_URL(realmName)}/user-management`

export const CONSOLE_IDENTITIES_URL = (realmName = ':realm_name') =>
  `${USER_MANAGEMENT_URL(realmName)}/identities`

export const CONSOLE_IDENTITY_URL = (realmName = ':realm_name', userId = ':user_id') =>
  `${CONSOLE_IDENTITIES_URL(realmName)}/${userId}`

export const CONSOLE_ORGANIZATIONS_URL = (realmName = ':realm_name') =>
  `${USER_MANAGEMENT_URL(realmName)}/organizations`

export const CONSOLE_ORGANIZATION_URL = (
  realmName = ':realm_name',
  organizationId = ':organizationId'
) => `${CONSOLE_ORGANIZATIONS_URL(realmName)}/${organizationId}`

export const CONSOLE_ROLES_URL = (realmName = ':realm_name') =>
  `${USER_MANAGEMENT_URL(realmName)}/roles`

export const CONSOLE_ROLE_URL = (realmName = ':realm_name', roleId = ':role_id') =>
  `${CONSOLE_ROLES_URL(realmName)}/${roleId}`
