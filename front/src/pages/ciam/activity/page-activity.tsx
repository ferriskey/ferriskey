import { Navigate, Route, Routes } from 'react-router'
import PageLiveFeature from './feature/page-live-feature'
import PageLogsFeature from './feature/page-logs-feature'
import PageMessagesFeature from './feature/page-messages-feature'
import PageSessionsFeature from './feature/page-sessions-feature'

export default function ConsoleActivity() {
  return (
    <Routes>
      <Route index element={<Navigate to='live' replace />} />
      <Route path='live' element={<PageLiveFeature />} />
      <Route path='logs' element={<PageLogsFeature />} />
      <Route path='sessions' element={<PageSessionsFeature />} />
      <Route path='messages' element={<PageMessagesFeature />} />
    </Routes>
  )
}
