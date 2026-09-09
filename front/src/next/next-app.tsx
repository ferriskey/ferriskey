import { Navigate, Route, Routes } from 'react-router'
import { NextAppShell } from './shell/app-shell'
import NextPageRole from './pages/role/page-role'

export default function NextApp() {
  return (
    <Routes>
      <Route element={<NextAppShell />}>
        <Route index element={<Navigate to='roles' replace />} />
        <Route path='roles/*' element={<NextPageRole />} />
      </Route>
    </Routes>
  )
}
