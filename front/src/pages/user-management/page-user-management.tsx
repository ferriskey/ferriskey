import { Navigate, Route, Routes } from 'react-router'
import PageIdentitiesFeature from './feature/page-identities-feature'

export default function PageUserManagement() {
  return (
    <Routes>
      <Route index element={<Navigate to='identities' replace />} />
      <Route path='identities' element={<PageIdentitiesFeature />} />
    </Routes>
  )
}
