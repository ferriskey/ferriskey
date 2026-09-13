import { Navigate, Route, Routes } from 'react-router'
import PageIdentitiesFeature from './feature/page-identities-feature'
import PageCreateIdentityFeature from './feature/page-create-identity-feature'
import PageIdentityDetailFeature from './feature/page-identity-detail-feature'
import PageOrganizationsFeature from './feature/page-organizations-feature'
import PageCreateOrganizationFeature from './feature/page-create-organization-feature'
import PageOrganizationDetailFeature from './feature/page-organization-detail-feature'
import PageRolesFeature from './feature/page-roles-feature'
import PageCreateRoleFeature from './feature/page-create-role-feature'
import PageRoleDetailFeature from './feature/page-role-detail-feature'

export default function ConsoleUserManagement() {
  return (
    <Routes>
      <Route index element={<Navigate to='identities' replace />} />

      <Route path='identities' element={<PageIdentitiesFeature />} />
      <Route path='identities/create' element={<PageCreateIdentityFeature />} />
      <Route path='identities/:user_id' element={<Navigate to='overview' replace />} />
      <Route path='identities/:user_id/*' element={<PageIdentityDetailFeature />} />

      <Route path='organizations' element={<PageOrganizationsFeature />} />
      <Route path='organizations/create' element={<PageCreateOrganizationFeature />} />
      <Route path='organizations/:organizationId' element={<Navigate to='settings' replace />} />
      <Route path='organizations/:organizationId/*' element={<PageOrganizationDetailFeature />} />

      <Route path='roles' element={<PageRolesFeature />} />
      <Route path='roles/create' element={<PageCreateRoleFeature />} />
      <Route path='roles/:role_id' element={<Navigate to='settings' replace />} />
      <Route path='roles/:role_id/*' element={<PageRoleDetailFeature />} />
    </Routes>
  )
}
