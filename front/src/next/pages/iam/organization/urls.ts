import { NEXT_ORGANIZATIONS_URL } from '@/next/routes'

export const NEXT_ORGANIZATION_URL = (
  realmName = ':realm_name',
  organizationId = ':organizationId'
) => `${NEXT_ORGANIZATIONS_URL(realmName)}/${organizationId}`
