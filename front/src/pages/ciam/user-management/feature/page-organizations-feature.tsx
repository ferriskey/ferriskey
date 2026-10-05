import { useNavigate, useParams } from 'react-router'
import { RouterParams } from '@/routes/router'
import { Schemas } from '@/api/api.client'
import PageOrganizationsOverview from '@/pages/iam/organization/ui/page-organizations-overview'
import { useOrganizationsOverview } from '@/pages/iam/organization/feature/use-organizations-overview'
import { CONSOLE_ORGANIZATION_URL, CONSOLE_ORGANIZATIONS_URL } from '../urls'

import Organization = Schemas.Organization

export default function PageOrganizationsFeature() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const realm = realm_name ?? 'master'

  const { firstDisabled, ...overview } = useOrganizationsOverview(realm)

  return (
    <PageOrganizationsOverview
      {...overview}
      organizationHref={(organization: Organization) =>
        `${CONSOLE_ORGANIZATION_URL(realm, organization.id)}/settings`
      }
      onCreate={() => navigate(`${CONSOLE_ORGANIZATIONS_URL(realm)}/create`)}
      onReviewDisabled={() => {
        if (!firstDisabled) return
        navigate(`${CONSOLE_ORGANIZATION_URL(realm, firstDisabled.id)}/settings`)
      }}
    />
  )
}
