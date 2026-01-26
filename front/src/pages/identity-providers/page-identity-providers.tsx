import { Route, Routes, Navigate } from 'react-router'
import PageOverviewFeature from './feature/page-overview-feature'
import PageCreateFeature from './feature/page-create-feature'
import PageDetailFeature from './feature/page-detail-feature'
import ProvidersLayout from './layouts/providers-layout'

export default function PageIdentityProviders() {
  return (
    <Routes>
      <Route index element={<Navigate to='overview' replace />} />

      <Route element={<ProvidersLayout />}>
        <Route path='overview' element={<PageOverviewFeature />} />
      </Route>

      <Route path='create' element={<PageCreateFeature />} />
      <Route path=':providerId' element={<PageDetailFeature />} />
    </Routes>
  )
}
