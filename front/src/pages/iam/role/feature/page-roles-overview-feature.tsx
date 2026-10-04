import { useNavigate, useParams } from 'react-router'
import { useGetRoles, useRoleCount, type RolesQuery } from '@/api/role.api'
import { usePagedListing } from '@/components/kit'
import { RouterParams } from '@/routes/router'
import { Schemas } from '@/api/api.client'
import { ROLE_URL, ROLES_URL } from '@/routes/router'
import PageRolesOverview from '../ui/page-roles-overview'

import Role = Schemas.Role

const ROLE_FILTER_KEYS = ['name', 'description', 'require_mfa', 'client_id'] as const

export default function PageRolesOverviewFeature() {
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
      roleHref={(role: Role) => `${ROLE_URL(realm, role.id)}/settings`}
      onCreate={() => navigate(`${ROLES_URL(realm)}/create`)}
    />
  )
}
