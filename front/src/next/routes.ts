import { REALM_URL } from '@/routes/router'

export const NEXT_URL = (realmName = ':realm_name') => `${REALM_URL(realmName)}/next`

export const NEXT_ROLES_URL = (realmName = ':realm_name') =>
  `${NEXT_URL(realmName)}/roles`
export const NEXT_ROLE_URL = (realmName = ':realm_name', roleId = ':role_id') =>
  `${NEXT_ROLES_URL(realmName)}/${roleId}`
