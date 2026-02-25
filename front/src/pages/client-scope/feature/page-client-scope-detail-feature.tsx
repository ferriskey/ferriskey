import { useGetClientScope } from '@/api/client-scope.api'
import { RouterParams } from '@/routes/router'
import { useParams } from 'react-router'
import PageClientScopeDetail from '../ui/page-client-scope-detail'

export default function PageClientScopeDetailFeature() {
  const { realm_name, scope_id } = useParams<RouterParams>()

  const { data: scope, isLoading } = useGetClientScope({
    realm: realm_name ?? 'master',
    scopeId: scope_id,
  })

  if (!scope) {
    return (
      <div className='text-sm text-muted-foreground'>
        {isLoading ? 'Loading client scope details...' : 'Client scope not found.'}
      </div>
    )
  }

  return <PageClientScopeDetail scope={scope} isLoading={isLoading} />
}
