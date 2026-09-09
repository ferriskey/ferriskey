import { ORGANIZATIONS_URL } from '@/routes/router'

export const ORGANIZATION_URL = (
  realmName = ':realm_name',
  organizationId = ':organizationId'
) => `${ORGANIZATIONS_URL(realmName)}/${organizationId}`
