import { PORTAL_URL } from '@/routes/router'

export const PORTAL_THEMES_URL = (realmName = ':realm_name') =>
  `${PORTAL_URL(realmName)}/themes`

export const PORTAL_THEME_URL = (realmName = ':realm_name', themeId = ':theme_id') =>
  `${PORTAL_THEMES_URL(realmName)}/${themeId}`

export const PORTAL_THEME_PAGE_URL = (
  realmName = ':realm_name',
  themeId = ':theme_id',
  pageType = ':page_type'
) => `${PORTAL_THEME_URL(realmName, themeId)}/pages/${pageType}`

export const PORTAL_LAYOUTS_URL = (realmName = ':realm_name') =>
  `${PORTAL_URL(realmName)}/layouts`

export const PORTAL_LAYOUT_URL = (realmName = ':realm_name', layoutId = ':layout_id') =>
  `${PORTAL_LAYOUTS_URL(realmName)}/${layoutId}`
