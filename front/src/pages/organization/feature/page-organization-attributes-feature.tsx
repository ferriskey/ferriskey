import {
  useDeleteOrganizationAttribute,
  useGetOrganizationAttributes,
  useUpsertOrganizationAttribute,
} from '@/api/organization.api'
import { RouterParams } from '@/routes/router'
import { useParams } from 'react-router'
import PageOrganizationAttributes from '../ui/page-organization-attributes'

export default function PageOrganizationAttributesFeature() {
  const { realm_name, organizationId } = useParams<RouterParams & { organizationId: string }>()

  const { data: attributes, isLoading } = useGetOrganizationAttributes({
    realm: realm_name,
    organizationId,
  })

  const { mutate: upsertAttribute } = useUpsertOrganizationAttribute()
  const { mutate: deleteAttribute } = useDeleteOrganizationAttribute()

  const handleUpsert = (key: string, value: string) => {
    if (!realm_name || !organizationId) return
    upsertAttribute({
      body: { value },
      path: { realm_name, organization_id: organizationId, key },
    })
  }

  const handleDelete = (key: string) => {
    if (!realm_name || !organizationId) return
    deleteAttribute({
      path: { realm_name, organization_id: organizationId, key },
    })
  }

  return (
    <PageOrganizationAttributes
      attributes={attributes ?? []}
      isLoading={isLoading}
      onUpsert={handleUpsert}
      onDelete={handleDelete}
    />
  )
}
