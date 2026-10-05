import { useNavigate, useParams } from 'react-router'
import {
  CLIENT_SCOPE_FILTER_KEYS,
  useClientScopeCount,
  useDeleteClientScope,
  useGetClientScopes,
  type ClientScopesQuery,
} from '@/api/client-scope.api'
import { RouterParams } from '@/routes/router'
import { usePagedListing } from '@/components/kit'
import { Schemas } from '@/api/api.client'
import { CLIENT_SCOPES_URL } from '@/routes/router'
import { clientScopeUrl } from '../urls'
import PageClientScopesOverview from '../ui/page-client-scopes-overview'

import ClientScope = Schemas.ClientScope

const ALERT_PREVIEW = 5

export default function PageClientScopesOverviewFeature() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const realm = realm_name ?? 'master'

  const listing = usePagedListing(CLIENT_SCOPE_FILTER_KEYS)
  const { data: response, isLoading } = useGetClientScopes({
    realm,
    query: listing.apiQuery as ClientScopesQuery,
    keepPrevious: true,
  })
  const total = useClientScopeCount({ realm })
  const defaults = useClientScopeCount({ realm, filter: { default_scope_type: 'DEFAULT' } })
  const optionals = useClientScopeCount({ realm, filter: { default_scope_type: 'OPTIONAL' } })
  const withMappers = useClientScopeCount({ realm, filter: { has_protocol_mappers: true } })
  const { data: withoutMappers } = useGetClientScopes({
    realm,
    query: { has_protocol_mappers: false, limit: ALERT_PREVIEW },
  })
  const { mutate: deleteClientScope, isPending: isDeleting } = useDeleteClientScope()

  return (
    <PageClientScopesOverview
      scopes={response?.data ?? []}
      pagination={response?.metadata}
      listing={listing}
      isLoading={isLoading}
      isDeleting={isDeleting}
      counts={{
        total: total.count,
        default: defaults.count,
        optional: optionals.count,
        withMappers: withMappers.count,
      }}
      withoutMappers={{
        total: withoutMappers?.metadata.total ?? 0,
        names: (withoutMappers?.data ?? []).map((scope) => scope.name),
      }}
      scopeHref={(scope: ClientScope) => `${clientScopeUrl(realm, scope.id)}/details`}
      onCreate={() => navigate(`${CLIENT_SCOPES_URL(realm)}/create`)}
      onDelete={(scope: ClientScope) =>
        deleteClientScope({ path: { realm_name: realm, scope_id: scope.id } })
      }
    />
  )
}
