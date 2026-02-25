import { REALM_URL } from '../router'

export const CLIENT_SCOPES_URL = (realmName = ':realmName') => `${REALM_URL(realmName)}/client-scopes`
export const CLIENT_SCOPE_URL = (realmName = ':realmName', scopeId = ':scopeId') =>
  `${CLIENT_SCOPES_URL(realmName)}/${scopeId}`
export const CLIENT_SCOPES_OVERVIEW_URL = '/overview'
export const CLIENT_SCOPES_CREATE_URL = '/create'
export const CLIENT_SCOPE_DETAILS_URL = '/details'
