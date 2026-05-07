import { Navigate, Route, Routes } from 'react-router'
import PageLiveActivityFeature from './feature/page-live-activity-feature'
import PageActivityComingSoon from './ui/page-activity-coming-soon'

export default function PageActivity() {
  return (
    <Routes>
      <Route index element={<Navigate to='live' replace />} />
      <Route path='live' element={<PageLiveActivityFeature />} />
      <Route
        path='logs'
        element={
          <PageActivityComingSoon
            title='Logs & events'
            description='Searchable feed of authentication events, errors and admin actions.'
          />
        }
      />
      <Route
        path='sessions'
        element={
          <PageActivityComingSoon
            title='Sessions'
            description='Active user sessions across devices, with revocation controls.'
          />
        }
      />
      <Route
        path='messages'
        element={
          <PageActivityComingSoon
            title='Message delivery'
            description='Outbound transactional emails and webhook deliveries.'
          />
        }
      />
    </Routes>
  )
}
