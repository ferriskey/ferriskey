import { useCreateRole } from '@/api/role.api'
import { RouterParams } from '@/routes/router'
import { UM_ROLES_URL } from '@/routes/sub-router/user-management.router'
import { useNavigate, useParams } from 'react-router'
import { toast } from 'sonner'
import PageCreateRole, { CreateRoleValues } from '../ui/page-create-role'

export default function PageCreateRoleFeature() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const { mutate, isPending } = useCreateRole()

  const handleCancel = () => {
    if (!realm_name) return
    navigate(UM_ROLES_URL(realm_name))
  }

  const handleSubmit = (values: CreateRoleValues) => {
    if (!realm_name) return
    mutate(
      {
        realmName: realm_name,
        body: {
          name: values.name,
          description: values.description,
          permissions: values.permissions,
        },
      },
      {
        onSuccess: () => {
          toast.success('Role created')
          navigate(UM_ROLES_URL(realm_name))
        },
        onError: () => toast.error('Failed to create role'),
      },
    )
  }

  return <PageCreateRole onCancel={handleCancel} onSubmit={handleSubmit} isSubmitting={isPending} />
}
