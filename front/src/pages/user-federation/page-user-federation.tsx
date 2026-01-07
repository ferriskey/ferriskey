import { Route, Routes } from 'react-router'
import PageOverviewFeature from './feature/page-overview-feature'
import PageCreateProviderFeature from './feature/page-create-provider-feature'
import UserFederationLayout from './layout/user-federation-layout'

export default function PageUserFederation() {
  return (
    <Routes>
      <Route element={<UserFederationLayout />}>
        <Route path='/overview' element={<PageOverviewFeature />} />
      </Route>
      <Route path='/create' element={<PageCreateProviderFeature />} />
    </Routes>
  )
}
