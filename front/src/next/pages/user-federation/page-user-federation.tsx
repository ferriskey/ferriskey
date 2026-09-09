import { Navigate, Route, Routes } from 'react-router'
import PageProvidersOverviewFeature from './feature/page-providers-overview-feature'
import PageCreateProviderFeature from './feature/page-create-provider-feature'
import PageProviderDetailFeature from './feature/page-provider-detail-feature'

export default function NextPageUserFederation() {
  return (
    <Routes>
      <Route index element={<PageProvidersOverviewFeature />} />
      <Route path='create' element={<PageCreateProviderFeature />} />
      <Route path=':provider_id' element={<Navigate to='settings' replace />} />
      <Route path=':provider_id/*' element={<PageProviderDetailFeature />} />
    </Routes>
  )
}
