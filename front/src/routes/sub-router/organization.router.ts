import { REALM_URL } from '../router'

export const ORGANIZATIONS_URL = (realmName = ':realmName') =>
  `${REALM_URL(realmName)}/organizations`

export const ORGANIZATION_URL = (
  realmName = ':realmName',
  organizationId = ':organizationId'
) => `${ORGANIZATIONS_URL(realmName)}/${organizationId}`

export const ORGANIZATION_OVERVIEW_URL = '/overview'
export const ORGANIZATION_CREATE_URL = '/create'
export const ORGANIZATION_SETTINGS_URL = '/settings'
export const ORGANIZATION_ATTRIBUTES_URL = '/attributes'
export const ORGANIZATION_MEMBERS_URL = '/members'

export type OrganizationRouterParams = {
  realm_name: string
  organizationId: string
}
