import { Route, Routes } from 'react-router'
import ClientScopesLayout from './layout/client-scopes-layout'
import PageClientScopesOverviewFeature from './feature/page-client-scopes-overview-feature'
import PageCreateClientScopeFeature from './feature/page-create-client-scope-feature'
import ClientScopeLayout from './layout/client-scope-layout'
import PageClientScopeDetailFeature from './feature/page-client-scope-detail-feature'
import PageProtocolMappersFeature from './feature/page-protocol-mappers-feature'
import PageCreateProtocolMapperFeature from './feature/page-create-protocol-mapper-feature'

export default function PageClientScope() {
  return (
    <Routes>
      <Route element={<ClientScopesLayout />}>
        <Route path='overview' element={<PageClientScopesOverviewFeature />} />
        <Route path='create' element={<PageCreateClientScopeFeature />} />
      </Route>
      <Route element={<ClientScopeLayout />}>
        <Route path=':scope_id/details' element={<PageClientScopeDetailFeature />} />
        <Route path=':scope_id/mappers' element={<PageProtocolMappersFeature />} />
        <Route path=':scope_id/mappers/new' element={<PageCreateProtocolMapperFeature />} />
      </Route>
    </Routes>
  )
}
