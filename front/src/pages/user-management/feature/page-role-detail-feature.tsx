import { useDeleteRole, useGetRole, useUpdateRole, useUpdateRolePermissions } from '@/api/role.api'
import { UM_ROLES_URL } from '@/routes/sub-router/user-management.router'
import { useNavigate, useParams } from 'react-router'
import { toast } from 'sonner'
import PageRoleDetail, { RoleDetailValues } from '../ui/page-role-detail'

type Params = {
  realm_name?: string
  role_id?: string
}

export default function PageRoleDetailFeature() {
  const { realm_name, role_id } = useParams<Params>()
  const navigate = useNavigate()
  const { data: roleResponse, isLoading } = useGetRole({ realm: realm_name, roleId: role_id })
  const { mutate: updateRole, isPending: isUpdatingRole } = useUpdateRole()
  const { mutate: updatePermissions, isPending: isUpdatingPerms } = useUpdateRolePermissions()
  const { mutateAsync: deleteRole, isPending: isDeleting } = useDeleteRole()

  const handleBack = () => {
    if (!realm_name) return
    navigate(UM_ROLES_URL(realm_name))
  }

  const handleSave = (values: RoleDetailValues) => {
    if (!realm_name || !role_id || !roleResponse) return
    const role = roleResponse.data
    const profileChanged =
      values.name !== role.name || (values.description ?? '') !== (role.description ?? '')
    const initialPerms = role.permissions ?? []
    const permsChanged =
      values.permissions.length !== initialPerms.length ||
      values.permissions.some((p) => !initialPerms.includes(p))

    if (profileChanged) {
      updateRole({
        path: { realm_name, role_id },
        body: { name: values.name, description: values.description || null },
      })
    }
    if (permsChanged) {
      updatePermissions({
        path: { realm_name, role_id },
        body: { permissions: values.permissions },
      })
    }
  }

  const handleDelete = async () => {
    if (!realm_name || !role_id) return
    try {
      await deleteRole({ path: { realm_name, role_id } })
      navigate(UM_ROLES_URL(realm_name))
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to delete role')
    }
  }

  return (
    <PageRoleDetail
      role={roleResponse?.data ?? null}
      isLoading={isLoading}
      isUpdating={isUpdatingRole || isUpdatingPerms}
      isDeleting={isDeleting}
      onBack={handleBack}
      onSave={handleSave}
      onDelete={handleDelete}
    />
  )
}
