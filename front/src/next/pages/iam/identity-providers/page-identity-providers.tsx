import { Route, Routes } from 'react-router'
import PageProvidersOverviewFeature from './feature/page-providers-overview-feature'
import PageCreateProviderFeature from './feature/page-create-provider-feature'
import PageProviderDetailFeature from './feature/page-provider-detail-feature'

export default function NextPageIdentityProviders() {
  return (
    <Routes>
      <Route index element={<PageProvidersOverviewFeature />} />
      <Route path='create' element={<PageCreateProviderFeature />} />
      <Route path=':alias' element={<PageProviderDetailFeature />} />
    </Routes>
  )
}
