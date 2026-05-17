import { REALM_URL } from '../router'

export const PORTAL_URL = (realmName = ':realmName') => `${REALM_URL(realmName)}/portal`
export const PORTAL_THEME_URL = (realmName = ':realmName') => `${PORTAL_URL(realmName)}/theme`
export const PORTAL_BUILDER_DEMO_URL = (realmName = ':realmName') =>
  `${PORTAL_URL(realmName)}/builder-demo`
