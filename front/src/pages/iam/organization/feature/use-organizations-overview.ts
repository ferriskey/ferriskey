import {
  ORGANIZATION_FILTER_KEYS,
  useGetOrganizations,
  useOrganizationCount,
  type OrganizationsQuery,
} from '@/api/organization.api'
import { usePagedListing } from '@/components/kit'

const ALERT_PREVIEW = 5

export function useOrganizationsOverview(realm: string) {
  const listing = usePagedListing(ORGANIZATION_FILTER_KEYS)
  const { data: response, isLoading } = useGetOrganizations({
    realm,
    query: listing.apiQuery as OrganizationsQuery,
    keepPrevious: true,
  })
  const total = useOrganizationCount({ realm })
  const enabled = useOrganizationCount({ realm, filter: { enabled: true } })
  const withDomain = useOrganizationCount({ realm, filter: { has_domain: true } })
  const { data: disabled } = useGetOrganizations({
    realm,
    query: { enabled: false, order_by: 'name', order: 'asc', limit: ALERT_PREVIEW },
  })
  const disabledOrganizations = disabled?.data ?? []
  const disabledTotal = disabled?.metadata.total ?? 0

  return {
    organizations: response?.data ?? [],
    pagination: response?.metadata,
    listing,
    isLoading,
    counts: {
      total: total.count,
      enabled: enabled.count,
      disabled: disabledTotal,
      withDomain: withDomain.count,
    },
    disabled: {
      total: disabledTotal,
      names: disabledOrganizations.map((organization) => organization.name),
    },
    firstDisabled: disabledOrganizations[0],
  }
}
