import { REALM_URL } from '../router'

export const PORTAL_URL = (realmName = ':realmName') => `${REALM_URL(realmName)}/portal`

// Collection (new in PR5).
export const PORTAL_THEMES_URL = (realmName = ':realmName') => `${PORTAL_URL(realmName)}/themes`
export const PORTAL_THEME_BUILDER_URL = (realmName = ':realmName', themeId = ':themeId') =>
  `${PORTAL_THEMES_URL(realmName)}/${themeId}`

// Legacy single-theme editor + sandbox demo (kept until cleanup PR).
export const PORTAL_THEME_URL = (realmName = ':realmName') => `${PORTAL_URL(realmName)}/theme`
export const PORTAL_BUILDER_DEMO_URL = (realmName = ':realmName') =>
  `${PORTAL_URL(realmName)}/builder-demo`

export type PortalThemeRouterParams = {
  realm_name: string
  theme_id: string
}
