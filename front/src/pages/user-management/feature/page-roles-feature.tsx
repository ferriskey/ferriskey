import { useGetRoles } from '@/api/role.api'
import { RouterParams } from '@/routes/router'
import { ROLE_OVERVIEW_URL, ROLE_URL } from '@/routes/sub-router/role.router'
import { UM_ROLE_CREATE_URL } from '@/routes/sub-router/user-management.router'
import { useNavigate, useParams } from 'react-router'
import PageRoles from '../ui/page-roles'

export default function PageRolesFeature() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const { data, isLoading } = useGetRoles({ realm: realm_name ?? 'master' })

  const handleSelect = (roleId: string) => {
    if (!realm_name) return
    navigate(`${ROLE_URL(realm_name, roleId)}${ROLE_OVERVIEW_URL}`)
  }

  const handleCreate = () => {
    if (!realm_name) return
    navigate(UM_ROLE_CREATE_URL(realm_name))
  }

  return (
    <PageRoles
      roles={data?.data ?? []}
      isLoading={isLoading}
      onSelect={handleSelect}
      onCreate={handleCreate}
    />
  )
}
