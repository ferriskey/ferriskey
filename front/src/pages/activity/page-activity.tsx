import ConsoleComingSoon from '@/components/console-coming-soon'
import { Navigate, Route, Routes } from 'react-router'
import PageLiveActivityFeature from './feature/page-live-activity-feature'
import PageLogsEventsFeature from './feature/page-logs-events-feature'

export default function PageActivity() {
  return (
    <Routes>
      <Route index element={<Navigate to='live' replace />} />
      <Route path='live' element={<PageLiveActivityFeature />} />
      <Route path='logs' element={<PageLogsEventsFeature />} />
      <Route
        path='sessions'
        element={
          <ConsoleComingSoon
            title='Sessions'
            description='Active user sessions across devices, with revocation controls.'
          />
        }
      />
      <Route
        path='messages'
        element={
          <ConsoleComingSoon
            title='Message delivery'
            description='Outbound transactional emails and webhook deliveries.'
          />
        }
      />
    </Routes>
  )
}
