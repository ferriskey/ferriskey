import { CLIENTS_URL } from '@/routes/router'

export const CLIENT_URL = (realmName = ':realm_name', clientId = ':client_id') =>
  `${CLIENTS_URL(realmName)}/${clientId}`

export const CLIENT_CREATE_URL = (realmName = ':realm_name') =>
  `${CLIENTS_URL(realmName)}/create`
