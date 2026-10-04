import { useNavigate, useParams } from 'react-router'
import { RouterParams } from '@/routes/router'
import { Schemas } from '@/api/api.client'
import { ORGANIZATIONS_URL } from '@/routes/router'
import { ORGANIZATION_URL } from '../urls'
import PageOrganizationsOverview from '../ui/page-organizations-overview'
import { useOrganizationsOverview } from './use-organizations-overview'

import Organization = Schemas.Organization

export default function PageOrganizationsOverviewFeature() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const realm = realm_name ?? 'master'

  const { firstDisabled, ...overview } = useOrganizationsOverview(realm)

  return (
    <PageOrganizationsOverview
      {...overview}
      organizationHref={(organization: Organization) =>
        `${ORGANIZATION_URL(realm, organization.id)}/settings`
      }
      onCreate={() => navigate(`${ORGANIZATIONS_URL(realm)}/create`)}
      onReviewDisabled={() => {
        if (!firstDisabled) return
        navigate(`${ORGANIZATION_URL(realm, firstDisabled.id)}/settings`)
      }}
    />
  )
}
