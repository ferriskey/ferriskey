import { REALM_URL } from "../router";

export const ROLE_URL = (realmName = ":realmName") => `${REALM_URL(realmName)}/roles`

export const ROLE_OVERVIEW_URL = '/overview'