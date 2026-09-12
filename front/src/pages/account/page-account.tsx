import { Route, Routes } from 'react-router'
import AccountLayout from './layouts/account-layout'
import PageAccountFeature from './feature/page-account-feature'
import PageAccountSessionsFeature from './feature/page-account-sessions-feature'

export default function PageAccountModule() {
  return (
    <Routes>
      <Route element={<AccountLayout />}>
        <Route index element={<PageAccountFeature />} />
        <Route path='sessions' element={<PageAccountSessionsFeature />} />
      </Route>
    </Routes>
  )
}
