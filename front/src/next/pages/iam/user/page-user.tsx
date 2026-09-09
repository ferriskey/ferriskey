import { Navigate, Route, Routes } from 'react-router'
import PageUsersOverviewFeature from './feature/page-users-overview-feature'
import PageCreateUserFeature from './feature/page-create-user-feature'
import PageUserDetailFeature from './feature/page-user-detail-feature'

export default function NextPageUsers() {
  return (
    <Routes>
      <Route index element={<PageUsersOverviewFeature />} />
      <Route path='create' element={<PageCreateUserFeature />} />
      <Route path=':user_id' element={<Navigate to='overview' replace />} />
      <Route path=':user_id/*' element={<PageUserDetailFeature />} />
    </Routes>
  )
}
