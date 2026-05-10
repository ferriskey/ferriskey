import { REALM_URL } from '../router'

export const APPLICATIONS_URL = (realmName = ':realmName') =>
  `${REALM_URL(realmName)}/console/applications`

export const APPLICATION_CREATE_URL = (realmName = ':realmName') =>
  `${APPLICATIONS_URL(realmName)}/create`

export const APPLICATION_CREATE_TYPE_URL = (
  realmName = ':realmName',
  type = ':type',
) => `${APPLICATION_CREATE_URL(realmName)}/${type}`

export type ApplicationType = 'native' | 'spa' | 'web' | 'm2m'
