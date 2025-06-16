import { useParams } from 'react-router'
import { RouterParams } from '@/routes/router.ts'
import { useGetRole } from '@/api/role.api.ts'
import PageRolePermissions from '@/pages/role/ui/page-role-permissions.tsx'

export default function PageRolePermissionsFeature() {
  const { realm_name, role_id } = useParams<RouterParams>()

  const { data: role } = useGetRole({
    realm: realm_name || 'master',
    roleId: role_id,
  })

  if (!role) return null

  return (
    <PageRolePermissions role={role} />
  )
}
