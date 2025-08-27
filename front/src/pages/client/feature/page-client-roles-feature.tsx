import { useParams } from 'react-router'
import { useGetClientRoles } from '@/api/client.api'
import { useDeleteRole } from '@/api/role.api'
import { Schemas } from '@/api/api.client'
import { ClientRouterParams } from '@/routes/sub-router/client.router'
import PageClientRoles from '../ui/page-client-roles'
import Role = Schemas.Role

export default function PageClientRolesFeature() {
  const { realm_name, client_id } = useParams<ClientRouterParams>()

  const { data: roles, isLoading, isError } = useGetClientRoles({
    realm: realm_name || 'master',
    clientId: client_id
  })

  const { mutate: deleteRole } = useDeleteRole()

  const handleDeleteRole = (role: Role) => {
    if (realm_name && client_id && role.id) {
      deleteRole({
        path: {
          realm_name,
          client_id,
          role_id: role.id,
        }
      })
    }
  }

  return (
    <PageClientRoles
      roles={roles?.data || []}
      isLoading={isLoading}
      isError={isError}
      clientId={client_id}
      handleDeleteRole={handleDeleteRole}
    />
  )
}
