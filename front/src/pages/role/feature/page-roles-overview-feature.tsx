import { RouterParams } from '@/routes/router'
import { useNavigate, useParams } from 'react-router'
import { useDeleteRole, useGetRoles } from '../../../api/role.api'
import PageRolesOverview from '../ui/page-roles-overview'
import { ROLE_SETTINGS_URL, ROLE_URL } from '@/routes/sub-router/role.router'
import { Schemas } from '@/api/api.client'
import Role = Schemas.Role

export default function PageRolesOverviewFeature() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const { data: rolesResponse, isLoading } = useGetRoles({ realm: realm_name ?? 'master' })
  const { mutate: deleteRole } = useDeleteRole()

  const handleDeleteSelected = (items: Role[]) => {
    if (!realm_name) return
    items.forEach((role) => {
      if (role.client?.id) {
        deleteRole(
          {
            path: {
              realm_name: realm_name ?? 'master',
              client_id: role.client.id,
              role_id: role.id,
            }
          }
        )
      }
    })
  }

  const handleDeleteRole = (role: Role) => {
    if (role.client?.id) {
      deleteRole({
        path: {
          realm_name: realm_name ?? 'master',
          client_id: role.client.id,
          role_id: role.id,
        }
      })
    }
  }

  const handleClickRow = (roleId: string) => {
    navigate(`${ROLE_URL(realm_name, roleId)}${ROLE_SETTINGS_URL}`)
  }



  return (
    <PageRolesOverview
      data={rolesResponse?.data || []}
      isLoading={isLoading}
      realmName={realm_name ?? 'master'}
      handleDeleteSelected={handleDeleteSelected}
      handleClickRow={handleClickRow}
      handleDeleteRole={handleDeleteRole}
    />
  )
}
