import { Navigate, Route, Routes } from 'react-router'
import { ConsoleShell } from './components/shell/ciam/console-shell'
import ConsoleActivity from './pages/ciam/activity/page-activity'
import ConsoleUserManagement from './pages/ciam/user-management/page-user-management'
import ConsoleApplications from './pages/ciam/applications/page-applications'
import ConsoleAuthentication from './pages/ciam/authentication/page-authentication'
import ConsoleBranding from './pages/ciam/branding/page-branding'

export default function ConsoleApp() {
  return (
    <Routes>
      <Route element={<ConsoleShell />}>
        <Route index element={<Navigate to='activity/live' replace />} />
        <Route path='activity/*' element={<ConsoleActivity />} />
        <Route path='user-management/*' element={<ConsoleUserManagement />} />
        <Route path='applications/*' element={<ConsoleApplications />} />
        <Route path='authentication/*' element={<ConsoleAuthentication />} />
        <Route path='branding/*' element={<ConsoleBranding />} />
      </Route>
    </Routes>
  )
}
