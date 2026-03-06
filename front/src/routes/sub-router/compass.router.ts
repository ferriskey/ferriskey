import { REALM_URL } from '../router'

export const COMPASS_OVERVIEW_URL = (realmName = ':realmName') =>
  `${REALM_URL(realmName)}/compass/overview`

export const COMPASS_FLOW_DETAIL_URL = (realmName = ':realmName', flowId = ':flow_id') =>
  `${REALM_URL(realmName)}/compass/${flowId}`
