import { Route, Routes } from 'react-router'
import PageSecurityEventsFeature from './feature/page-security-events-feature'

export default function NextPageSeaWatch() {
  return (
    <Routes>
      <Route index element={<PageSecurityEventsFeature />} />
    </Routes>
  )
}
