import { Navigate, Route, Routes } from 'react-router'
import PageRealmSettingsFeature from './feature/page-realm-settings-feature'

export default function NextPageRealmSettings() {
  return (
    <Routes>
      <Route index element={<Navigate to='general' replace />} />
      <Route path='*' element={<PageRealmSettingsFeature />} />
    </Routes>
  )
}
