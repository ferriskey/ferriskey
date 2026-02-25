import { Route, Routes } from 'react-router-dom'
import PageClientsOverviewFeature from './feature/page-clients-overview-feature'
import Container from './container'
import ClientLayout from './layout/client-layout'
import PageClientSettingsFeature from './feature/page-client-settings-feature'
import PageClientCredentialsFeature from './feature/page-client-credentials-feature'
import PageCreateClientFeature from '@/pages/client/feature/page-create-client-feature.tsx'
import PageClientRolesFeature from './feature/page-client-roles-feature'

export default function PageClient() {
  return (
    <Routes>
      <Route element={<Container />}>
        <Route path='/overview' element={<PageClientsOverviewFeature />} />
        <Route path='/create' element={<PageCreateClientFeature />} />
      </Route>

      <Route element={<ClientLayout />}>
        <Route path='/:client_id/settings' element={<PageClientSettingsFeature />} />
        <Route path='/:client_id/credentials' element={<PageClientCredentialsFeature />} />
        <Route path='/:client_id/roles' element={<PageClientRolesFeature />} />
      </Route>
    </Routes>
  )
}
