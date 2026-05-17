import { useCreateOrganization } from '@/api/organization.api'
import { RouterParams } from '@/routes/router'
import { UM_ORGANIZATIONS_URL } from '@/routes/sub-router/user-management.router'
import { useNavigate, useParams } from 'react-router'
import { toast } from 'sonner'
import PageCreateOrganization from '../ui/page-create-organization'

export default function PageCreateOrganizationFeature() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const { mutate, isPending } = useCreateOrganization()

  const handleCancel = () => {
    if (!realm_name) return
    navigate(UM_ORGANIZATIONS_URL(realm_name))
  }

  const handleSubmit = (values: { name: string; alias: string; domain: string; description: string }) => {
    if (!realm_name) return
    mutate(
      {
        path: { realm_name },
        body: {
          name: values.name,
          alias: values.alias,
          domain: values.domain || null,
          description: values.description || null,
          enabled: true,
        },
      },
      {
        onSuccess: () => {
          toast.success('Organization created')
          navigate(UM_ORGANIZATIONS_URL(realm_name))
        },
        onError: () => toast.error('Failed to create organization'),
      },
    )
  }

  return <PageCreateOrganization onCancel={handleCancel} onSubmit={handleSubmit} isSubmitting={isPending} />
}
