import { useGetOrganizations } from '@/api/organization.api'
import { RouterParams } from '@/routes/router'
import {
  UM_ORGANIZATION_CREATE_URL,
  UM_ORGANIZATION_URL,
} from '@/routes/sub-router/user-management.router'
import { useNavigate, useParams } from 'react-router'
import PageOrganizations from '../ui/page-organizations'

export default function PageOrganizationsFeature() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const { data, isLoading } = useGetOrganizations({ realm: realm_name ?? 'master' })

  const handleSelect = (organizationId: string) => {
    if (!realm_name) return
    navigate(UM_ORGANIZATION_URL(realm_name, organizationId))
  }

  const handleCreate = () => {
    if (!realm_name) return
    navigate(UM_ORGANIZATION_CREATE_URL(realm_name))
  }

  return (
    <PageOrganizations
      organizations={data?.data ?? []}
      isLoading={isLoading}
      onSelect={handleSelect}
      onCreate={handleCreate}
    />
  )
}
