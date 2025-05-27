import { REALM_URL } from "../router";

export const ROLES_URL = (realmName = ":realmName") => `${REALM_URL(realmName)}/roles`
export const ROLE_URL = (realmName = ":realmName", roleId = ":roleId") => `${ROLES_URL(realmName)}/${roleId}`

export const ROLE_OVERVIEW_URL = '/overview'
export const ROLE_SETTINGS_URL = '/settings'