import {
  FEDERATION_PROVIDER_FILTER_KEYS,
  useFederationProviderCount,
  useGetUserFederations,
  type FederationProvidersQuery,
} from '@/api/user-federation.api'
import { usePagedListing } from '@/components/kit'

const ALERT_PREVIEW = 5

const NEVER_SYNCED_PREVIEW: FederationProvidersQuery = {
  synced: false,
  order_by: 'name',
  order: 'asc',
  limit: ALERT_PREVIEW,
}

export function useProvidersOverview(realm: string) {
  const listing = usePagedListing(FEDERATION_PROVIDER_FILTER_KEYS)
  const { data: response, isLoading } = useGetUserFederations({
    realm,
    query: listing.apiQuery as FederationProvidersQuery,
    keepPrevious: true,
  })
  const total = useFederationProviderCount({ realm })
  const enabled = useFederationProviderCount({ realm, filter: { enabled: true } })
  const disabled = useFederationProviderCount({ realm, filter: { enabled: false } })
  const scheduled = useFederationProviderCount({ realm, filter: { sync_enabled: true } })
  const { data: neverSynced } = useGetUserFederations({ realm, query: NEVER_SYNCED_PREVIEW })

  return {
    providers: response?.data ?? [],
    pagination: response?.metadata,
    listing,
    isLoading,
    counts: {
      total: total.count,
      enabled: enabled.count,
      disabled: disabled.count,
      scheduled: scheduled.count,
    },
    neverSynced: {
      total: neverSynced?.metadata.total ?? 0,
      names: (neverSynced?.data ?? []).map((provider) => provider.name),
    },
  }
}
