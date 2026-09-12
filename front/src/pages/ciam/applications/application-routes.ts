import { CONSOLE_URL } from '@/routes/router'

export const CONSOLE_APPLICATIONS_URL = (realmName = ':realm_name') =>
  `${CONSOLE_URL(realmName)}/applications`

export const CONSOLE_APPLICATION_CREATE_URL = (realmName = ':realm_name') =>
  `${CONSOLE_APPLICATIONS_URL(realmName)}/create`

export const CONSOLE_APPLICATION_PICKER_URL = (realmName = ':realm_name') =>
  `${CONSOLE_APPLICATIONS_URL(realmName)}?create=1`

export const CONSOLE_APPLICATION_URL = (
  realmName = ':realm_name',
  clientId = ':client_id'
) => `${CONSOLE_APPLICATIONS_URL(realmName)}/${clientId}`
