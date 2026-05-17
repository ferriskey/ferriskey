import { PORTAL_URL } from './portal-theme.router'

export const PORTAL_LAYOUTS_URL = (realmName = ':realmName') =>
  `${PORTAL_URL(realmName)}/layouts`

export const PORTAL_LAYOUT_BUILDER_URL = (realmName = ':realmName', layoutId = ':layoutId') =>
  `${PORTAL_LAYOUTS_URL(realmName)}/${layoutId}`

export type PortalLayoutRouterParams = {
  realm_name: string
  layout_id: string
}
