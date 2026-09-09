import { NEXT_CLIENTS_URL } from '@/next/routes'

export const NEXT_CLIENT_URL = (realmName = ':realm_name', clientId = ':client_id') =>
  `${NEXT_CLIENTS_URL(realmName)}/${clientId}`

export const NEXT_CLIENT_CREATE_URL = (realmName = ':realm_name') =>
  `${NEXT_CLIENTS_URL(realmName)}/create`
