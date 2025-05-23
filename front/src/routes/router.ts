export const REALM_URL = (realmName = ':realmName') => `/realms/${realmName}`
export const REALM_OVERVIEW_URL = '/overview'

export type RouterParams = {
  realm_name: string
}