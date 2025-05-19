import { REALM_URL } from "../router";

export const USER_URL = (realmName = ":realmName") => `${REALM_URL(realmName)}/users`

export const USER_OVERVIEW_URL = '/overview'