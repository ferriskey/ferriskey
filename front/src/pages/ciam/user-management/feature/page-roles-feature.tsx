import { useMemo } from 'react'
import { useNavigate, useParams } from 'react-router'
import { useGetRoles } from '@/api/role.api'
import { RouterParams } from '@/routes/router'
import { Schemas } from '@/api/api.client'
import PageRolesOverview from '@/pages/iam/role/ui/page-roles-overview'
import { CONSOLE_ROLE_URL, CONSOLE_ROLES_URL } from '../urls'

import Role = Schemas.Role

export default function PageRolesFeature() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const realm = realm_name ?? 'master'

  const { data: rolesResponse, isLoading } = useGetRoles({ realm })
  const roles = useMemo(() => rolesResponse?.data ?? [], [rolesResponse])

  return (
    <PageRolesOverview
      roles={roles}
      isLoading={isLoading}
      roleHref={(role: Role) => `${CONSOLE_ROLE_URL(realm, role.id)}/settings`}
      onCreate={() => navigate(`${CONSOLE_ROLES_URL(realm)}/create`)}
    />
  )
}
