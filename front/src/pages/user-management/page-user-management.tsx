import { Navigate, Route, Routes } from 'react-router'
import PageCreateOrganizationFeature from './feature/page-create-organization-feature'
import PageCreateRoleFeature from './feature/page-create-role-feature'
import PageIdentitiesFeature from './feature/page-identities-feature'
import PageOrganizationsFeature from './feature/page-organizations-feature'
import PageRolesFeature from './feature/page-roles-feature'

export default function PageUserManagement() {
  return (
    <Routes>
      <Route index element={<Navigate to='identities' replace />} />
      <Route path='identities' element={<PageIdentitiesFeature />} />
      <Route path='organizations' element={<PageOrganizationsFeature />} />
      <Route path='organizations/create' element={<PageCreateOrganizationFeature />} />
      <Route path='roles' element={<PageRolesFeature />} />
      <Route path='roles/create' element={<PageCreateRoleFeature />} />
    </Routes>
  )
}
