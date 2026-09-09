import { Navigate, Route, Routes } from 'react-router'
import { NextAppShell } from './shell/app-shell'
import NextPageRole from './pages/role/page-role'
import NextPageOverview from './pages/overview/page-overview'
import NextPageClients from './pages/client/page-client'
import NextPageUsers from './pages/user/page-user'
import NextPageClientScopes from './pages/client-scope/page-client-scope'
import NextPageOrganizations from './pages/organization/page-organization'
import NextPageRealmSettings from './pages/realm/page-realm'
import NextPagePortal from './pages/portal/page-portal'
import NextPageEmailTemplates from './pages/email-template/page-email-template'
import NextPageWebhooks from './pages/webhook/page-webhook'
import NextPageIdentityProviders from './pages/identity-providers/page-identity-providers'
import NextPageUserFederation from './pages/user-federation/page-user-federation'
import NextPageSeaWatch from './pages/seawatch/page-seawatch'
import NextPageCompass from './pages/compass/page-compass'
import NextPageAccount from './pages/account/page-account'

export default function NextApp() {
  return (
    <Routes>
      <Route element={<NextAppShell />}>
        <Route index element={<Navigate to='overview' replace />} />
        <Route path='roles/*' element={<NextPageRole />} />
        <Route path='overview/*' element={<NextPageOverview />} />
        <Route path='clients/*' element={<NextPageClients />} />
        <Route path='users/*' element={<NextPageUsers />} />
        <Route path='client-scopes/*' element={<NextPageClientScopes />} />
        <Route path='organizations/*' element={<NextPageOrganizations />} />
        <Route path='realm-settings/*' element={<NextPageRealmSettings />} />
        <Route path='portal/*' element={<NextPagePortal />} />
        <Route path='email-templates/*' element={<NextPageEmailTemplates />} />
        <Route path='webhooks/*' element={<NextPageWebhooks />} />
        <Route path='identity-providers/*' element={<NextPageIdentityProviders />} />
        <Route path='user-federation/*' element={<NextPageUserFederation />} />
        <Route path='seawatch/*' element={<NextPageSeaWatch />} />
        <Route path='compass/*' element={<NextPageCompass />} />
        <Route path='account/*' element={<NextPageAccount />} />
      </Route>
    </Routes>
  )
}
