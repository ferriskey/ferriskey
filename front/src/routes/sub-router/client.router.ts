import { REALM_URL } from '../router'

export const CLIENTS_URL = (realmName = ':realmName') => `${REALM_URL(realmName)}/clients`
export const CLIENT_URL = (realmName = ':realmName', clientId = ':clientId') =>  `${CLIENTS_URL(realmName)}/${clientId}`

export const CLIENT_OVERVIEW_URL = (realmName = ':realm_name', clientId = ':client_id') => `${CLIENTS_URL(realmName)}/${clientId}${SETTINGS_URL}`

export const OVERVIEW_URL = '/overview'
export const SETTINGS_URL = '/settings'
export const CLIENT_CREATE_URL = '/create'

export type ClientRouterParams = {
  realm_name: string
  client_id: string
  current_view: 'settings' | 'credentials' | 'roles'
}
