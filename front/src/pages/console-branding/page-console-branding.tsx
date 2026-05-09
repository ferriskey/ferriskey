import ConsoleComingSoon from '@/components/console-coming-soon'
import { Navigate, Route, Routes } from 'react-router'

export default function PageConsoleBranding() {
  return (
    <Routes>
      <Route index element={<Navigate to='email-templates' replace />} />
      <Route
        path='email-templates'
        element={
          <ConsoleComingSoon
            title='Email templates'
            description='Customize the look and content of transactional emails sent to customers.'
          />
        }
      />
    </Routes>
  )
}
