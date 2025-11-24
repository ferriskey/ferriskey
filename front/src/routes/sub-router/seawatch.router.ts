import { REALM_URL } from '../router'

export const SEAWATCH_OVERVIEW_URL = (realmName = ':realmName') =>
  `${REALM_URL(realmName)}/seawatch/overview`
