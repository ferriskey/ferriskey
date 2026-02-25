import { useParams } from 'react-router-dom'
import { useGetClientRoles } from '@/api/client.api'
import { useDeleteRole } from '@/api/role.api'
import { Schemas } from '@/api/api.client'
import { ClientRouterParams } from '@/routes/sub-router/client.router'
import PageClientRoles from '../ui/page-client-roles'
import Role = Schemas.Role

export default function PageClientRolesFeature() {
  const { realm_name, client_id } = useParams<ClientRouterParams>()

  const {
    data: roles,
    isLoading,
    isError,
    refetch,
  } = useGetClientRoles({
    realm: realm_name || 'master',
    clientId: client_id,
  })

  const { mutateAsync: deleteRole } = useDeleteRole()

  const handleDeleteRole = async (role: Role) => {
    if (realm_name && role.id) {
      await deleteRole({
        path: {
          realm_name,
          role_id: role.id,
        },
      })
      await refetch()
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
