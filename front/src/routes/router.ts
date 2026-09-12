export const REALM_URL = (realmName = ':realm_name') => `/realms/${realmName}`
export const CONSOLE_URL = (realmName = ':realm_name') => `${REALM_URL(realmName)}/console`

export const ROLES_URL = (realmName = ':realm_name') =>
  `${REALM_URL(realmName)}/roles`
export const ROLE_URL = (realmName = ':realm_name', roleId = ':role_id') =>
  `${ROLES_URL(realmName)}/${roleId}`

export const OVERVIEW_URL = (realmName = ':realm_name') =>
  `${REALM_URL(realmName)}/overview`

export const CLIENTS_URL = (realmName = ':realm_name') =>
  `${REALM_URL(realmName)}/clients`

export const USERS_URL = (realmName = ':realm_name') =>
  `${REALM_URL(realmName)}/users`

export const CLIENT_SCOPES_URL = (realmName = ':realm_name') =>
  `${REALM_URL(realmName)}/client-scopes`

export const ORGANIZATIONS_URL = (realmName = ':realm_name') =>
  `${REALM_URL(realmName)}/organizations`

export const REALM_SETTINGS_URL = (realmName = ':realm_name') =>
  `${REALM_URL(realmName)}/realm-settings`

export const PORTAL_URL = (realmName = ':realm_name') =>
  `${REALM_URL(realmName)}/portal`

export const EMAIL_TEMPLATES_URL = (realmName = ':realm_name') =>
  `${REALM_URL(realmName)}/email-templates`

export const WEBHOOKS_URL = (realmName = ':realm_name') =>
  `${REALM_URL(realmName)}/webhooks`

export const IDENTITY_PROVIDERS_URL = (realmName = ':realm_name') =>
  `${REALM_URL(realmName)}/identity-providers`

export const USER_FEDERATION_URL = (realmName = ':realm_name') =>
  `${REALM_URL(realmName)}/user-federation`

export const SEAWATCH_URL = (realmName = ':realm_name') =>
  `${REALM_URL(realmName)}/seawatch`

export const COMPASS_URL = (realmName = ':realm_name') =>
  `${REALM_URL(realmName)}/compass`

export const ACCOUNT_URL = (realmName = ':realm_name') =>
  `${REALM_URL(realmName)}/account`


export type RouterParams = {
  realm_name: string
  client_id?: string
  user_id?: string
  role_id?: string
  scope_id?: string
  mapper_id?: string
  webhook_id?: string
  template_id?: string
}
