import { Route, Routes } from 'react-router'
import PageUsersOverviewFeature from './feature/page-users-overview-feature'
import UserLayout from './layouts/user-layout'
import UsersLayout from './layouts/users-layout'
import PageUserOverviewFeature from './feature/page-user-overview-feature.tsx'
import PageCredentialFeature from './feature/page-credential-feature'
import PageUserRoleMappingFeature from './feature/page-user-role-mapping-feature.tsx'

export default function PageUser() {
  return (
    <Routes>
      <Route element={<UsersLayout />}>
        <Route path='/overview' element={<PageUsersOverviewFeature />} />
      </Route>

      <Route element={<UserLayout />}>
        <Route path='/:user_id/overview' element={<PageUserOverviewFeature />} />
        <Route path='/:user_id/credentials' element={<PageCredentialFeature />} />
        <Route path='/:user_id/role-mapping' element={<PageUserRoleMappingFeature />} />
      </Route>
    </Routes>
  )
}
