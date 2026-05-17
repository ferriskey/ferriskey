import { REALM_URL } from '../router'

export const ACTIVITY_URL = (realmName = ':realmName') => `${REALM_URL(realmName)}/console/activity`
export const ACTIVITY_LIVE_URL = (realmName = ':realmName') => `${ACTIVITY_URL(realmName)}/live`
export const ACTIVITY_LOGS_URL = (realmName = ':realmName') => `${ACTIVITY_URL(realmName)}/logs`
export const ACTIVITY_SESSIONS_URL = (realmName = ':realmName') => `${ACTIVITY_URL(realmName)}/sessions`
export const ACTIVITY_MESSAGES_URL = (realmName = ':realmName') => `${ACTIVITY_URL(realmName)}/messages`
