export const REALM_URL = (realmName: string) => `/realms/${realmName}`

export interface RouterParams {
  realm_name: string
}