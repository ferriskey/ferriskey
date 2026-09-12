import { Route, Routes } from 'react-router'
import PageAccountFeature from './feature/page-account-feature'
import PageAccountSessionsFeature from './feature/page-account-sessions-feature'

export default function PageAccount() {
  return (
    <Routes>
      <Route index element={<PageAccountFeature />} />
      <Route path='sessions' element={<PageAccountSessionsFeature />} />
    </Routes>
  )
}
