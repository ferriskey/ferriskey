import { Navigate, Route, Routes } from 'react-router'
import PageApplicationsListFeature from './feature/page-applications-list-feature'
import PageCreateApplicationFeature from './feature/page-create-application-feature'
import PageApplicationDetailFeature from './feature/page-application-detail-feature'

export default function ConsoleApplications() {
  return (
    <Routes>
      <Route index element={<PageApplicationsListFeature />} />
      <Route path='create' element={<PageCreateApplicationFeature />} />
      <Route path=':client_id' element={<Navigate to='quickstart' replace />} />
      <Route path=':client_id/*' element={<PageApplicationDetailFeature />} />
    </Routes>
  )
}
