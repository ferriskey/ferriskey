import { useMemo } from 'react'
import { useNavigate, useParams } from 'react-router'
import { useGetOrganizations } from '@/api/organization.api'
import { RouterParams } from '@/routes/router'
import { Schemas } from '@/api/api.client'
import { NEXT_ORGANIZATIONS_URL } from '@/next/routes'
import { NEXT_ORGANIZATION_URL } from '../urls'
import PageOrganizationsOverview from '../ui/page-organizations-overview'

import Organization = Schemas.Organization

export default function PageOrganizationsOverviewFeature() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const realm = realm_name ?? 'master'

  const { data, isLoading } = useGetOrganizations({ realm })
  const organizations = useMemo(() => data?.data ?? [], [data])

  const firstDisabled = organizations.find((o) => !o.enabled)

  return (
    <PageOrganizationsOverview
      organizations={organizations}
      isLoading={isLoading}
      organizationHref={(organization: Organization) =>
        `${NEXT_ORGANIZATION_URL(realm, organization.id)}/settings`
      }
      onCreate={() => navigate(`${NEXT_ORGANIZATIONS_URL(realm)}/create`)}
      onReviewDisabled={() => {
        if (!firstDisabled) return
        navigate(`${NEXT_ORGANIZATION_URL(realm, firstDisabled.id)}/settings`)
      }}
    />
  )
}
