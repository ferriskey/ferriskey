import { REALM_URL } from '../router'

export const PORTAL_URL = (realmName = ':realmName') => `${REALM_URL(realmName)}/portal`

// Collection (new in PR5).
export const PORTAL_THEMES_URL = (realmName = ':realmName') => `${PORTAL_URL(realmName)}/themes`
export const PORTAL_THEME_BUILDER_URL = (realmName = ':realmName', themeId = ':themeId') =>
  `${PORTAL_THEMES_URL(realmName)}/${themeId}`

// Builder section URLs — every tab is addressable so it can be shared via
// link or restored after a refresh.
export type PortalThemeBuilderSection = 'theme' | 'layout'
export const PORTAL_THEME_BUILDER_SECTION_URL = (
  realmName = ':realmName',
  themeId = ':themeId',
  section: PortalThemeBuilderSection = 'theme',
) => `${PORTAL_THEME_BUILDER_URL(realmName, themeId)}/${section}`

export const PORTAL_THEME_BUILDER_PAGE_URL = (
  realmName = ':realmName',
  themeId = ':themeId',
  pageType = ':page_type',
) => `${PORTAL_THEME_BUILDER_URL(realmName, themeId)}/pages/${pageType}`

// Legacy single-theme editor + sandbox demo (kept until cleanup PR).
export const PORTAL_THEME_URL = (realmName = ':realmName') => `${PORTAL_URL(realmName)}/theme`
export const PORTAL_BUILDER_DEMO_URL = (realmName = ':realmName') =>
  `${PORTAL_URL(realmName)}/builder-demo`

// Console (CIAM) — themes are reachable from the Branding section and render
// full-viewport (outside the console ProductLayout chrome).
export const CONSOLE_THEMES_URL = (realmName = ':realmName') =>
  `${REALM_URL(realmName)}/console/branding/themes`
export const CONSOLE_THEME_BUILDER_URL = (realmName = ':realmName', themeId = ':themeId') =>
  `${CONSOLE_THEMES_URL(realmName)}/${themeId}`

/**
 * Theme URLs scoped to the current context: the console branding section in
 * CIAM mode, otherwise the admin portal. Lets the shared themes list/builder
 * components keep navigation within the panel they were opened from.
 */
const inConsole = (pathname: string) => pathname.includes('/console/')

export const themesListUrl = (pathname: string, realmName = ':realmName') =>
  inConsole(pathname) ? CONSOLE_THEMES_URL(realmName) : PORTAL_THEMES_URL(realmName)

export const themeBuilderUrl = (
  pathname: string,
  realmName = ':realmName',
  themeId = ':themeId',
) =>
  inConsole(pathname)
    ? CONSOLE_THEME_BUILDER_URL(realmName, themeId)
    : PORTAL_THEME_BUILDER_URL(realmName, themeId)

export const themeBuilderSectionUrl = (
  pathname: string,
  realmName = ':realmName',
  themeId = ':themeId',
  section: PortalThemeBuilderSection = 'theme',
) =>
  inConsole(pathname)
    ? `${CONSOLE_THEME_BUILDER_URL(realmName, themeId)}/${section}`
    : PORTAL_THEME_BUILDER_SECTION_URL(realmName, themeId, section)

export const themeBuilderPageUrl = (
  pathname: string,
  realmName = ':realmName',
  themeId = ':themeId',
  pageType = ':page_type',
) =>
  inConsole(pathname)
    ? `${CONSOLE_THEME_BUILDER_URL(realmName, themeId)}/pages/${pageType}`
    : PORTAL_THEME_BUILDER_PAGE_URL(realmName, themeId, pageType)

export type PortalThemeRouterParams = {
  realm_name: string
  theme_id: string
}
