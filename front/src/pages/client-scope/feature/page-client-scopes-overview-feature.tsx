import { useGetClientScopes } from '@/api/client-scope.api'
import { RouterParams } from '@/routes/router'
import { useMemo } from 'react'
import { useNavigate, useParams } from 'react-router'
import {
  CLIENT_SCOPE_DETAILS_URL,
  CLIENT_SCOPE_URL,
} from '@/routes/sub-router/client-scope.router'
import PageClientScopesOverview from '../ui/page-client-scopes-overview'

export default function PageClientScopesOverviewFeature() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const { data: response, isLoading } = useGetClientScopes({ realm: realm_name ?? 'master' })

  const scopes = useMemo(() => response?.data ?? [], [response])

  const statistics = useMemo(() => {
    const totalScopes = scopes.length
    const defaultScopes = scopes.filter((scope) => scope.is_default).length
    const optionalScopes = scopes.filter((scope) => !scope.is_default).length
    const withProtocolMappers = scopes.filter((scope) => (scope.protocol_mappers?.length ?? 0) > 0).length

    return { totalScopes, defaultScopes, optionalScopes, withProtocolMappers }
  }, [scopes])

  const handleClickRow = (scopeId: string) => {
    navigate(`${CLIENT_SCOPE_URL(realm_name, scopeId)}${CLIENT_SCOPE_DETAILS_URL}`)
  }

  return (
    <PageClientScopesOverview
      data={scopes}
      isLoading={isLoading}
      statistics={statistics}
      handleClickRow={handleClickRow}
    />
  )
}
