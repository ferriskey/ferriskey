import { Navigate, Route, Routes } from 'react-router'
import { NextAppShell } from './shell/iam/app-shell'
import NextConsoleApp from './console-app'
import NextPageRole from './pages/iam/role/page-role'
import NextPageOverview from './pages/iam/overview/page-overview'
import NextPageClients from './pages/iam/client/page-client'
import NextPageUsers from './pages/iam/user/page-user'
import NextPageClientScopes from './pages/iam/client-scope/page-client-scope'
import NextPageOrganizations from './pages/iam/organization/page-organization'
import NextPageRealmSettings from './pages/iam/realm/page-realm'
import NextPagePortal from './pages/iam/portal/page-portal'
import NextPageEmailTemplates from './pages/iam/email-template/page-email-template'
import NextPageWebhooks from './pages/iam/webhook/page-webhook'
import NextPageIdentityProviders from './pages/iam/identity-providers/page-identity-providers'
import NextPageUserFederation from './pages/iam/user-federation/page-user-federation'
import NextPageSeaWatch from './pages/iam/seawatch/page-seawatch'
import NextPageCompass from './pages/iam/compass/page-compass'
import NextPageAccount from './pages/iam/account/page-account'

export default function NextApp() {
  return (
    <Routes>
      <Route path='console/*' element={<NextConsoleApp />} />

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
