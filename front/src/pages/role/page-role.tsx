import { Route, Routes } from 'react-router'
import PageRolesOverviewFeature from './feature/page-roles-overview-feature'
import PageRoleSettingsFeature from './feature/page-role-settings-feature'
import RoleLayout from './layout/role-layout'
import RolesLayout from './layout/roles-layout'
import PageRolePermissionsFeature from './feature/page-role-permissions-feature'
import PageRoleUsersFeature from './feature/page-role-users-feature'
import PageCreateRoleFeature from './feature/page-create-role-feature'

export default function PageRole() {
  return (
    <Routes>
      <Route element={<RolesLayout />}>
        <Route path='/overview' element={<PageRolesOverviewFeature />} />
        <Route path='/create' element={<PageCreateRoleFeature />} />
      </Route>
      <Route element={<RoleLayout />}>
        <Route path='/:role_id/settings' element={<PageRoleSettingsFeature />} />
        <Route path='/:role_id/permissions' element={<PageRolePermissionsFeature />} />
        <Route path='/:role_id/users' element={<PageRoleUsersFeature />} />
      </Route>
    </Routes>
  )
}
