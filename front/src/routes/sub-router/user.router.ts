import { REALM_URL } from "../router";

export const USERS_URL = (realmName = ":realmName") => `${REALM_URL(realmName)}/users`
export const USER_URL = (realmName = ":realmName", userId = ":userId") => `${USERS_URL(realmName)}/${userId}`


export const USER_OVERVIEW_URL = '/overview'

export type UserRouterParams = {
  realm_name: string
  user_id: string
  current_view: 'overview' | 'credentials' | 'role-mapping'
}