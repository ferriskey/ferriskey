import { Route, Routes } from 'react-router'
import PageOverviewFeature from './feature/page-overview-feature'
import UserFederationLayout from './layout/user-federation-layout'

export default function PageUserFederation() {
  return (
    <Routes>
      <Route element={<UserFederationLayout />}>
        <Route path='/overview' element={<PageOverviewFeature />} />
      </Route>
    </Routes>
  )
}
