import { useParams } from 'react-router'
import { useGetUserRoles } from '@/api/user.api'
import { UserRouterParams } from '@/routes/sub-router/user.router'
import PageUserRoleMapping from '../ui/page-user-role-mapping'
import { useUnassignUserRole } from '@/api/user_role.api'
import { useEffect } from 'react'
import { toast } from 'sonner'

export default function PageUserRoleMappingFeature() {
  const { realm_name, user_id } = useParams<UserRouterParams>()

  const {
    data: userRoles,
    isLoading,
    isError,
  } = useGetUserRoles({
    realm: realm_name || 'master',
    userId: user_id || '',
  })
  const { mutate: unassignRole, isSuccess } = useUnassignUserRole()

  const handleUnassignRole = (roleId: string) => {
    if (!realm_name || !user_id) return
    unassignRole({
      path: {
        realm_name,
        role_id: roleId,
        user_id,
      }
    })
  }

  useEffect(() => {
    if (isSuccess) {
      toast.success('Role unassigned successfully', {
        description: 'The role has been successfully removed from the user.',
      })
    }
  }, [isSuccess])

  return (
    <PageUserRoleMapping
      userRoles={userRoles?.data || []}
      isLoading={isLoading}
      isError={isError}
      userId={user_id}
      handleUnassignRole={handleUnassignRole}
    />
  )
}
