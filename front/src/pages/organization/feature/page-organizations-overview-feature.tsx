import { useGetOrganizations } from '@/api/organization.api'
import { RouterParams } from '@/routes/router'
import {
  ORGANIZATIONS_URL,
  ORGANIZATION_CREATE_URL,
  ORGANIZATION_SETTINGS_URL,
  ORGANIZATION_URL,
} from '@/routes/sub-router/organization.router'
import { useNavigate, useParams } from 'react-router'
import PageOrganizationsOverview from '../ui/page-organizations-overview'

export default function PageOrganizationsOverviewFeature() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()

  const { data, isLoading } = useGetOrganizations({ realm: realm_name ?? 'master' })

  return (
    <PageOrganizationsOverview
      data={data?.data ?? []}
      isLoading={isLoading}
      onRowClick={(org) =>
        navigate(`${ORGANIZATION_URL(realm_name, org.id)}${ORGANIZATION_SETTINGS_URL}`)
      }
      onCreateClick={() =>
        navigate(`${ORGANIZATIONS_URL(realm_name)}${ORGANIZATION_CREATE_URL}`)
      }
    />
  )
}
