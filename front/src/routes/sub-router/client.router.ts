import { REALM_URL } from "../router";

export const CLIENTS_URL = (realmName = ":realmName") => `${REALM_URL(realmName)}/clients`
export const CLIENT_URL = (realmName = ":realmName", clientId = ":clientId") =>  `${CLIENTS_URL(realmName)}/${clientId}`

export const CLIENT_OVERVIEW_URL = (realmNam = ":realm_name", clientId = ":client_id") => `${CLIENTS_URL(realmNam)}/${clientId}${SETTINGS_URL}`

export const OVERVIEW_URL = '/overview'
export const SETTINGS_URL = '/settings'
export const CLIENT_CREATE_URL = '/create'
