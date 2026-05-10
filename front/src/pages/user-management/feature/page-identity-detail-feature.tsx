import { useGetRoles } from '@/api/role.api'
import { useBulkDeleteUser, useGetUser, useGetUserCredentials, useGetUserRoles, useUpdateUser } from '@/api/user.api'
import { useAssignUserRole, useUnassignUserRole } from '@/api/user_role.api'
import { RouterParams } from '@/routes/router'
import { IDENTITIES_URL } from '@/routes/sub-router/user-management.router'
import { useNavigate, useParams } from 'react-router'
import { toast } from 'sonner'
import PageIdentityDetail, { IdentityProfileValues } from '../ui/page-identity-detail'

export default function PageIdentityDetailFeature() {
  const { realm_name, user_id } = useParams<RouterParams>()
  const navigate = useNavigate()
  const { data: userResponse, isLoading } = useGetUser({ realm: realm_name, userId: user_id })
  const { data: credentialsResponse } = useGetUserCredentials({ realm: realm_name, userId: user_id })
  const { data: rolesResponse } = useGetUserRoles({ realm: realm_name, userId: user_id ?? '' })
  const { data: allRolesResponse } = useGetRoles({ realm: realm_name ?? 'master' })
  const { mutate: updateUser, isPending: isUpdating } = useUpdateUser()
  const { mutateAsync: deleteUser, isPending: isDeleting } = useBulkDeleteUser()
  const { mutateAsync: assignRole, isPending: isAssigning } = useAssignUserRole()
  const { mutateAsync: unassignRole, isPending: isUnassigning } = useUnassignUserRole()

  const handleBack = () => {
    if (!realm_name) return
    navigate(IDENTITIES_URL(realm_name))
  }

  const handleSave = (values: IdentityProfileValues) => {
    if (!realm_name || !user_id || !userResponse) return
    updateUser(
      {
        path: { realm_name, user_id },
        body: {
          firstname: values.firstname || undefined,
          lastname: values.lastname || undefined,
          email: values.email || undefined,
          email_verified: values.emailVerified,
          enabled: values.enabled,
          required_actions: values.requiredActions,
        },
      },
      {
        onSuccess: () => toast.success('Identity updated'),
        onError: (err) => toast.error(err.message ?? 'Failed to update identity'),
      },
    )
  }

  const handleAssignRole = async (roleId: string) => {
    if (!realm_name || !user_id) return
    try {
      await assignRole({ path: { realm_name, user_id, role_id: roleId } })
      toast.success('Role assigned')
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to assign role')
    }
  }

  const handleUnassignRole = async (roleId: string) => {
    if (!realm_name || !user_id) return
    try {
      await unassignRole({ path: { realm_name, user_id, role_id: roleId } })
      toast.success('Role removed')
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to remove role')
    }
  }

  const handleDelete = async () => {
    if (!realm_name || !user_id) return
    try {
      await deleteUser({ path: { realm_name }, body: { ids: [user_id] } })
      toast.success('Identity deleted')
      navigate(IDENTITIES_URL(realm_name))
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to delete identity')
    }
  }

  return (
    <PageIdentityDetail
      identity={userResponse?.data ?? null}
      credentials={credentialsResponse?.data ?? []}
      roles={rolesResponse?.data ?? []}
      availableRoles={allRolesResponse?.data ?? []}
      isLoading={isLoading}
      isUpdating={isUpdating}
      isDeleting={isDeleting}
      isMutatingRoles={isAssigning || isUnassigning}
      onBack={handleBack}
      onSave={handleSave}
      onDelete={handleDelete}
      onAssignRole={handleAssignRole}
      onUnassignRole={handleUnassignRole}
    />
  )
}
