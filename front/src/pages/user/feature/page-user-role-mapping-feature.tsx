import { useParams } from 'react-router'
import { useGetUserRoles } from '@/api/user.api'
import { UserRouterParams } from '@/routes/sub-router/user.router'
import PageUserRoleMapping from '../ui/page-user-role-mapping'

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

  const handleUnassignRole = (roleId: string) => {
    console.log(`Unassigning role with ID: ${roleId}`)
  }

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
