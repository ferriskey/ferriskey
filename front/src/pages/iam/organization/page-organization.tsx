import { Navigate, Route, Routes } from 'react-router'
import PageOrganizationsOverviewFeature from './feature/page-organizations-overview-feature'
import PageCreateOrganizationFeature from './feature/page-create-organization-feature'
import PageOrganizationDetailFeature from './feature/page-organization-detail-feature'

export default function PageOrganizations() {
  return (
    <Routes>
      <Route index element={<PageOrganizationsOverviewFeature />} />
      <Route path='create' element={<PageCreateOrganizationFeature />} />
      <Route path=':organizationId' element={<Navigate to='settings' replace />} />
      <Route path=':organizationId/*' element={<PageOrganizationDetailFeature />} />
    </Routes>
  )
}
