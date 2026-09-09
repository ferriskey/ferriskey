import { useMemo } from 'react'
import { useNavigate, useParams } from 'react-router'
import { useGetRoles } from '@/api/role.api'
import { RouterParams } from '@/routes/router'
import { Schemas } from '@/api/api.client'
import { NEXT_ROLE_URL, NEXT_ROLES_URL } from '@/next/routes'
import PageRolesOverview from '../ui/page-roles-overview'

import Role = Schemas.Role

export default function PageRolesOverviewFeature() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const realm = realm_name ?? 'master'

  const { data: rolesResponse, isLoading } = useGetRoles({ realm })
  const roles = useMemo(() => rolesResponse?.data ?? [], [rolesResponse])

  return (
    <PageRolesOverview
      roles={roles}
      isLoading={isLoading}
      roleHref={(role: Role) => `${NEXT_ROLE_URL(realm, role.id)}/settings`}
      onCreate={() => navigate(`${NEXT_ROLES_URL(realm)}/create`)}
    />
  )
}
