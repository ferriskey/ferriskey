import { Navigate, Route, Routes, useParams } from 'react-router'
import RealmsSettingsLayout from './layouts/realm-settings-layout'
import { REALM_SETTINGS_URL, RouterParams } from '@/routes/router'
import PageRealmSettingsGeneralFeature from './feature/page-realm-settings-general-feature'
import PageRealmSettingsLoginFeature from './feature/page-realm-settings-login-feature'
import PageRealmSettingsSecurityFeature from './feature/page-realm-settings-security-feature'

export default function PageRealm() {
  const { realm_name } = useParams<RouterParams>()
  return (
    <Routes>
      <Route element={<RealmsSettingsLayout />}>
        <Route index element={<PageRealmSettingsGeneralFeature />} />
        <Route path='/general' element={<PageRealmSettingsGeneralFeature />} />
        <Route path='/login' element={<PageRealmSettingsLoginFeature />} />
        <Route path='/security' element={<PageRealmSettingsSecurityFeature />} />
      </Route>
      <Route path='*' element={<Navigate to={REALM_SETTINGS_URL(realm_name)}/>} />
    </Routes>
  )
}
