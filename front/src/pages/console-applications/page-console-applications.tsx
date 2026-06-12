import { Route, Routes } from 'react-router'
import PageApplicationDetailFeature from './feature/page-application-detail-feature'
import PageApplicationsListFeature from './feature/page-applications-list-feature'
import PageCreateApplicationFeature from './feature/page-create-application-feature'
import PageCreateApplicationTypeFeature from './feature/page-create-application-type-feature'

export default function PageConsoleApplications() {
  return (
    <Routes>
      <Route index element={<PageApplicationsListFeature />} />
      <Route path='create' element={<PageCreateApplicationTypeFeature />} />
      <Route path='create/:type' element={<PageCreateApplicationFeature />} />
      <Route path=':client_id' element={<PageApplicationDetailFeature />} />
    </Routes>
  )
}
