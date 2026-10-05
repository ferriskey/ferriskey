import { useNavigate, useParams } from 'react-router'
import {
  ROLE_FILTER_KEYS,
  useGetRoles,
  useRoleCount,
  type RolesQuery,
} from '@/api/role.api'
import { usePagedListing } from '@/components/kit'
import { RouterParams } from '@/routes/router'
import { Schemas } from '@/api/api.client'
import { ROLE_URL, ROLES_URL } from '@/routes/router'
import PageRolesOverview from '../ui/page-roles-overview'

import Role = Schemas.Role

const EMPTY_PREVIEW = 5

export default function PageRolesOverviewFeature() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const realm = realm_name ?? 'master'

  const listing = usePagedListing(ROLE_FILTER_KEYS)
  const { data: rolesResponse, isLoading } = useGetRoles({
    realm,
    query: listing.apiQuery as RolesQuery,
    keepPrevious: true,
  })
  const total = useRoleCount({ realm })
  const realmRoles = useRoleCount({ realm, filter: { scope: 'realm' } })
  const clientRoles = useRoleCount({ realm, filter: { scope: 'client' } })
  const granting = useRoleCount({ realm, filter: { has_permissions: true } })
  const { data: emptyResponse } = useGetRoles({
    realm,
    query: { has_permissions: false, limit: EMPTY_PREVIEW },
  })

  return (
    <PageRolesOverview
      roles={rolesResponse?.data ?? []}
      pagination={rolesResponse?.metadata}
      listing={listing}
      isLoading={isLoading}
      counts={{
        total: total.count,
        realm: realmRoles.count,
        client: clientRoles.count,
        granting: granting.count,
      }}
      withoutPermissions={{
        total: emptyResponse?.metadata.total ?? 0,
        names: (emptyResponse?.data ?? []).map((role) => role.name),
      }}
      roleHref={(role: Role) => `${ROLE_URL(realm, role.id)}/settings`}
      onCreate={() => navigate(`${ROLES_URL(realm)}/create`)}
    />
  )
}
