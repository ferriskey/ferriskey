import { Navigate, Route, Routes } from 'react-router'
import PageClientsOverviewFeature from './feature/page-clients-overview-feature'
import PageCreateClientFeature from './feature/page-create-client-feature'
import PageClientDetailFeature from './feature/page-client-detail-feature'

export default function PageClients() {
  return (
    <Routes>
      <Route index element={<PageClientsOverviewFeature />} />
      <Route path='create' element={<PageCreateClientFeature />} />
      <Route path=':client_id' element={<Navigate to='settings' replace />} />
      <Route path=':client_id/*' element={<PageClientDetailFeature />} />
    </Routes>
  )
}
