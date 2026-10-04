import {
  IDENTITY_PROVIDER_FILTER_KEYS,
  useGetIdentityProviders,
  useIdentityProviderCount,
  type IdentityProvidersQuery,
} from '@/api/identity-providers.api'
import { usePagedListing } from '@/components/kit'
import { providerName } from '../provider-status'

const ALERT_PREVIEW = 5

const previewQuery = (health: 'error' | 'degraded'): IdentityProvidersQuery => ({
  health,
  order_by: 'alias',
  order: 'asc',
  limit: ALERT_PREVIEW,
})

export function useProvidersOverview(realm: string) {
  const listing = usePagedListing(IDENTITY_PROVIDER_FILTER_KEYS)
  const { data: response, isLoading } = useGetIdentityProviders({
    realm,
    query: listing.apiQuery as IdentityProvidersQuery,
  })
  const total = useIdentityProviderCount({ realm })
  const enabled = useIdentityProviderCount({ realm, filter: { enabled: true } })
  const disabled = useIdentityProviderCount({ realm, filter: { enabled: false } })
  const { data: broken } = useGetIdentityProviders({ realm, query: previewQuery('error') })
  const { data: degraded } = useGetIdentityProviders({ realm, query: previewQuery('degraded') })

  return {
    providers: response?.data ?? [],
    pagination: response?.metadata,
    listing,
    isLoading,
    counts: {
      total: total.count,
      enabled: enabled.count,
      disabled: disabled.count,
    },
    broken: {
      total: broken?.metadata.total ?? 0,
      names: (broken?.data ?? []).map(providerName),
    },
    degraded: {
      total: degraded?.metadata.total ?? 0,
      names: (degraded?.data ?? []).map(providerName),
    },
  }
}
