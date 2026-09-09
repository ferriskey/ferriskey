import { Navigate, Route, Routes } from 'react-router'
import { AppShell } from './components/shell/iam/app-shell'
import ConsoleApp from './console-app'
import PageRole from './pages/iam/role/page-role'
import PageOverview from './pages/iam/overview/page-overview'
import PageClients from './pages/iam/client/page-client'
import PageUsers from './pages/iam/user/page-user'
import PageClientScopes from './pages/iam/client-scope/page-client-scope'
import PageOrganizations from './pages/iam/organization/page-organization'
import PageRealmSettings from './pages/iam/realm/page-realm'
import PagePortal from './pages/iam/portal/page-portal'
import PageEmailTemplates from './pages/iam/email-template/page-email-template'
import PageWebhooks from './pages/iam/webhook/page-webhook'
import PageIdentityProviders from './pages/iam/identity-providers/page-identity-providers'
import PageUserFederation from './pages/iam/user-federation/page-user-federation'
import PageSeaWatch from './pages/iam/seawatch/page-seawatch'
import PageCompass from './pages/iam/compass/page-compass'
import PageAccount from './pages/iam/account/page-account'

export default function IamApp() {
  return (
    <Routes>
      <Route path='console/*' element={<ConsoleApp />} />

      <Route element={<AppShell />}>
        <Route index element={<Navigate to='overview' replace />} />
        <Route path='roles/*' element={<PageRole />} />
        <Route path='overview/*' element={<PageOverview />} />
        <Route path='clients/*' element={<PageClients />} />
        <Route path='users/*' element={<PageUsers />} />
        <Route path='client-scopes/*' element={<PageClientScopes />} />
        <Route path='organizations/*' element={<PageOrganizations />} />
        <Route path='realm-settings/*' element={<PageRealmSettings />} />
        <Route path='portal/*' element={<PagePortal />} />
        <Route path='email-templates/*' element={<PageEmailTemplates />} />
        <Route path='webhooks/*' element={<PageWebhooks />} />
        <Route path='identity-providers/*' element={<PageIdentityProviders />} />
        <Route path='user-federation/*' element={<PageUserFederation />} />
        <Route path='seawatch/*' element={<PageSeaWatch />} />
        <Route path='compass/*' element={<PageCompass />} />
        <Route path='account/*' element={<PageAccount />} />
      </Route>
    </Routes>
  )
}
