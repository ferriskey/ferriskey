import { REALM_URL } from '@/routes/router'

export const NEXT_URL = (realmName = ':realm_name') => `${REALM_URL(realmName)}/next`

export const NEXT_ROLES_URL = (realmName = ':realm_name') =>
  `${NEXT_URL(realmName)}/roles`
export const NEXT_ROLE_URL = (realmName = ':realm_name', roleId = ':role_id') =>
  `${NEXT_ROLES_URL(realmName)}/${roleId}`

export const NEXT_OVERVIEW_URL = (realmName = ':realm_name') =>
  `${NEXT_URL(realmName)}/overview`

export const NEXT_CLIENTS_URL = (realmName = ':realm_name') =>
  `${NEXT_URL(realmName)}/clients`

export const NEXT_USERS_URL = (realmName = ':realm_name') =>
  `${NEXT_URL(realmName)}/users`

export const NEXT_CLIENT_SCOPES_URL = (realmName = ':realm_name') =>
  `${NEXT_URL(realmName)}/client-scopes`

export const NEXT_ORGANIZATIONS_URL = (realmName = ':realm_name') =>
  `${NEXT_URL(realmName)}/organizations`

export const NEXT_REALM_SETTINGS_URL = (realmName = ':realm_name') =>
  `${NEXT_URL(realmName)}/realm-settings`

export const NEXT_PORTAL_URL = (realmName = ':realm_name') =>
  `${NEXT_URL(realmName)}/portal`

export const NEXT_EMAIL_TEMPLATES_URL = (realmName = ':realm_name') =>
  `${NEXT_URL(realmName)}/email-templates`

export const NEXT_WEBHOOKS_URL = (realmName = ':realm_name') =>
  `${NEXT_URL(realmName)}/webhooks`

export const NEXT_IDENTITY_PROVIDERS_URL = (realmName = ':realm_name') =>
  `${NEXT_URL(realmName)}/identity-providers`

export const NEXT_USER_FEDERATION_URL = (realmName = ':realm_name') =>
  `${NEXT_URL(realmName)}/user-federation`

export const NEXT_SEAWATCH_URL = (realmName = ':realm_name') =>
  `${NEXT_URL(realmName)}/seawatch`

export const NEXT_COMPASS_URL = (realmName = ':realm_name') =>
  `${NEXT_URL(realmName)}/compass`

export const NEXT_ACCOUNT_URL = (realmName = ':realm_name') =>
  `${NEXT_URL(realmName)}/account`
