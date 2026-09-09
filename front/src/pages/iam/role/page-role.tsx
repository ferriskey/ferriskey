import { Navigate, Route, Routes } from 'react-router'
import PageRolesOverviewFeature from './feature/page-roles-overview-feature'
import PageRoleDetailFeature from './feature/page-role-detail-feature'
import PageCreateRoleFeature from './feature/page-create-role-feature'

export default function PageRole() {
  return (
    <Routes>
      <Route index element={<PageRolesOverviewFeature />} />
      <Route path='create' element={<PageCreateRoleFeature />} />
      <Route path=':role_id' element={<Navigate to='settings' replace />} />
      <Route path=':role_id/*' element={<PageRoleDetailFeature />} />
    </Routes>
  )
}
