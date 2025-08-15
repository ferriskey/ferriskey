export const REALM_URL = (realmName = ':realmName') => `/realms/${realmName}`
export const REALM_OVERVIEW_URL = '/overview'

export type RouterParams = {
  realm_name: string
  client_id?: string
  user_id?: string
  role_id?: string
}
