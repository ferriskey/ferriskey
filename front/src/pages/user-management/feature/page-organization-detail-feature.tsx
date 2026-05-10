import {
  useDeleteOrganization,
  useGetOrganization,
  useGetOrganizationMembers,
  useUpdateOrganization,
} from '@/api/organization.api'
import { RouterParams } from '@/routes/router'
import { UM_ORGANIZATIONS_URL } from '@/routes/sub-router/user-management.router'
import { useNavigate, useParams } from 'react-router'
import { toast } from 'sonner'
import PageOrganizationDetail, {
  OrganizationDetailValues,
} from '../ui/page-organization-detail'

interface Params extends RouterParams {
  organization_id: string
}

export default function PageOrganizationDetailFeature() {
  const { realm_name, organization_id } = useParams<Params>()
  const navigate = useNavigate()
  const { data: orgResponse, isLoading } = useGetOrganization({
    realm: realm_name,
    organizationId: organization_id,
  })
  const { data: membersResponse } = useGetOrganizationMembers({
    realm: realm_name,
    organizationId: organization_id,
  })
  const { mutate: updateOrganization, isPending: isUpdating } = useUpdateOrganization()
  const { mutateAsync: deleteOrganization, isPending: isDeleting } = useDeleteOrganization()

  const handleBack = () => {
    if (!realm_name) return
    navigate(UM_ORGANIZATIONS_URL(realm_name))
  }

  const handleSave = (values: OrganizationDetailValues) => {
    if (!realm_name || !organization_id) return
    updateOrganization(
      {
        path: { realm_name, organization_id },
        body: {
          name: values.name,
          alias: values.alias,
          domain: values.domain || null,
          description: values.description || null,
          enabled: values.enabled,
        },
      },
      {
        onError: (err) => toast.error(err.message ?? 'Failed to update organization'),
      },
    )
  }

  const handleDelete = async () => {
    if (!realm_name || !organization_id) return
    try {
      await deleteOrganization({ path: { realm_name, organization_id } })
      navigate(UM_ORGANIZATIONS_URL(realm_name))
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to delete organization')
    }
  }

  return (
    <PageOrganizationDetail
      organization={orgResponse ?? null}
      memberCount={membersResponse?.length ?? 0}
      isLoading={isLoading}
      isUpdating={isUpdating}
      isDeleting={isDeleting}
      onBack={handleBack}
      onSave={handleSave}
      onDelete={handleDelete}
    />
  )
}
