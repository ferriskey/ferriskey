import { REALM_URL } from "../router";

export const CLIENT_URL = (realmName = ":realmName") => `${REALM_URL(realmName)}/clients`

export const CLIENT_OVERVIEW_URL = '/overview'