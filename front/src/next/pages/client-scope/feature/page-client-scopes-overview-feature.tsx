import { useMemo } from 'react'
import { useNavigate, useParams } from 'react-router'
import { useDeleteClientScope, useGetClientScopes } from '@/api/client-scope.api'
import { RouterParams } from '@/routes/router'
import { Schemas } from '@/api/api.client'
import { NEXT_CLIENT_SCOPES_URL } from '@/next/routes'
import { clientScopeUrl } from '../urls'
import PageClientScopesOverview from '../ui/page-client-scopes-overview'

import ClientScope = Schemas.ClientScope

export default function PageClientScopesOverviewFeature() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const realm = realm_name ?? 'master'

  const { data: response, isLoading } = useGetClientScopes({ realm })
  const { mutate: deleteClientScope, isPending: isDeleting } = useDeleteClientScope()

  const scopes = useMemo(() => response?.data ?? [], [response])

  return (
    <PageClientScopesOverview
      scopes={scopes}
      isLoading={isLoading}
      isDeleting={isDeleting}
      scopeHref={(scope: ClientScope) => `${clientScopeUrl(realm, scope.id)}/details`}
      onCreate={() => navigate(`${NEXT_CLIENT_SCOPES_URL(realm)}/create`)}
      onDelete={(scope: ClientScope) =>
        deleteClientScope({ path: { realm_name: realm, scope_id: scope.id } })
      }
    />
  )
}
