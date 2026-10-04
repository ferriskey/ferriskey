import { useNavigate, useParams } from 'react-router'
import { useGetRoles, useRoleCount, type RolesQuery } from '@/api/role.api'
import { usePagedListing } from '@/components/kit'
import { RouterParams } from '@/routes/router'
import { Schemas } from '@/api/api.client'
import PageRolesOverview from '@/pages/iam/role/ui/page-roles-overview'
import { CONSOLE_ROLE_URL, CONSOLE_ROLES_URL } from '../urls'

import Role = Schemas.Role

const ROLE_FILTER_KEYS = ['name', 'description', 'require_mfa', 'client_id'] as const

export default function PageRolesFeature() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const realm = realm_name ?? 'master'

  const listing = usePagedListing(ROLE_FILTER_KEYS)
  const { data: rolesResponse, isLoading } = useGetRoles({
    realm,
    query: listing.apiQuery as RolesQuery,
  })
  const total = useRoleCount({ realm })
  const mfa = useRoleCount({ realm, filter: { require_mfa: true } })

  return (
    <PageRolesOverview
      roles={rolesResponse?.data ?? []}
      pagination={rolesResponse?.metadata}
      listing={listing}
      isLoading={isLoading}
      counts={{ total: total.count, mfa: mfa.count }}
      roleHref={(role: Role) => `${CONSOLE_ROLE_URL(realm, role.id)}/settings`}
      onCreate={() => navigate(`${CONSOLE_ROLES_URL(realm)}/create`)}
    />
  )
}
