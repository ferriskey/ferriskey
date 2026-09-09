import { NEXT_PORTAL_URL } from '@/next/routes'

export const NEXT_PORTAL_THEMES_URL = (realmName = ':realm_name') =>
  `${NEXT_PORTAL_URL(realmName)}/themes`

export const NEXT_PORTAL_THEME_URL = (realmName = ':realm_name', themeId = ':theme_id') =>
  `${NEXT_PORTAL_THEMES_URL(realmName)}/${themeId}`

export const NEXT_PORTAL_THEME_PAGE_URL = (
  realmName = ':realm_name',
  themeId = ':theme_id',
  pageType = ':page_type'
) => `${NEXT_PORTAL_THEME_URL(realmName, themeId)}/pages/${pageType}`

export const NEXT_PORTAL_LAYOUTS_URL = (realmName = ':realm_name') =>
  `${NEXT_PORTAL_URL(realmName)}/layouts`

export const NEXT_PORTAL_LAYOUT_URL = (realmName = ':realm_name', layoutId = ':layout_id') =>
  `${NEXT_PORTAL_LAYOUTS_URL(realmName)}/${layoutId}`
