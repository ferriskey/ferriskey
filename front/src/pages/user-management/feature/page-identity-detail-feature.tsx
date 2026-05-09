import { useBulkDeleteUser, useGetUser, useGetUserCredentials, useGetUserRoles, useUpdateUser } from '@/api/user.api'
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
  const { data: rolesResponse } = useGetUserRoles({ realm: realm_name, userId: user_id })
  const { mutate: updateUser, isPending: isUpdating } = useUpdateUser()
  const { mutateAsync: deleteUser, isPending: isDeleting } = useBulkDeleteUser()

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
          username: userResponse.data.username,
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
      isLoading={isLoading}
      isUpdating={isUpdating}
      isDeleting={isDeleting}
      onBack={handleBack}
      onSave={handleSave}
      onDelete={handleDelete}
    />
  )
}
