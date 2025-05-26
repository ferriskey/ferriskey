import { REALM_URL } from "../router";

export const CLIENTS_URL = (realmName = ":realmName") => `${REALM_URL(realmName)}/clients`
export const CLIENT_OVERVIEW_URL = (realmNam = ":realm_name", clientId = ":client_id") => `${CLIENTS_URL(realmNam)}/${clientId}${OVERVIEW_URL}`

export const OVERVIEW_URL = '/overview'