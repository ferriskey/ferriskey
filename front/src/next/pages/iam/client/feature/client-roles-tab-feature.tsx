import { useGetClientRoles } from '@/api/client.api'
import { useDeleteRole } from '@/api/role.api'
import { Schemas } from '@/api/api.client'
import ClientRolesTab from '../ui/client-roles-tab'

import Client = Schemas.Client
import Role = Schemas.Role

export interface ClientRolesTabFeatureProps {
  client: Client
  realm: string
}

export default function ClientRolesTabFeature({ client, realm }: ClientRolesTabFeatureProps) {
  const {
    data: rolesResponse,
    isLoading,
    isError,
    refetch,
  } = useGetClientRoles({ realm, clientId: client.id })

  const { mutateAsync: deleteRole } = useDeleteRole()

  const handleDeleteRole = async (role: Role) => {
    await deleteRole({ path: { realm_name: realm, role_id: role.id } })
    await refetch()
  }

  return (
    <ClientRolesTab
      roles={rolesResponse?.data ?? []}
      isLoading={isLoading}
      isError={isError}
      onDeleteRole={(role) => void handleDeleteRole(role)}
    />
  )
}
