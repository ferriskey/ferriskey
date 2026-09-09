import { Navigate, Route, Routes } from 'react-router'
import PageClientScopesOverviewFeature from './feature/page-client-scopes-overview-feature'
import PageCreateClientScopeFeature from './feature/page-create-client-scope-feature'
import PageClientScopeDetailFeature from './feature/page-client-scope-detail-feature'
import PageCreateProtocolMapperFeature from './feature/page-create-protocol-mapper-feature'
import PageProtocolMapperSettingsFeature from './feature/page-protocol-mapper-settings-feature'

export default function NextPageClientScopes() {
  return (
    <Routes>
      <Route index element={<PageClientScopesOverviewFeature />} />
      <Route path='create' element={<PageCreateClientScopeFeature />} />
      <Route path=':scope_id' element={<Navigate to='details' replace />} />
      <Route path=':scope_id/mappers/new' element={<PageCreateProtocolMapperFeature />} />
      <Route path=':scope_id/mappers/:mapper_id' element={<PageProtocolMapperSettingsFeature />} />
      <Route path=':scope_id/*' element={<PageClientScopeDetailFeature />} />
    </Routes>
  )
}
