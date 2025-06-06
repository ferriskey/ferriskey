import { Route, Routes } from "react-router";
import PageClientsOverviewFeature from "./feature/page-clients-overview-feature";
import Container from "./container";
import ClientLayout from "./layout/client-layout";
import PageClientSettingsFeature from "./feature/page-client-settings-feature";
import PageClientCredentialsFeature from "./feature/page-client-credentials-feature";

export default function PageClient() {
  return (
    <Routes>
      <Route element={<Container />}>
        <Route path="/overview" element={<PageClientsOverviewFeature />} />
      </Route>    

      <Route element={<ClientLayout />}>
        <Route path="/:client_id/settings" element={<PageClientSettingsFeature />} />
        <Route path="/:client_id/credentials" element={<PageClientCredentialsFeature />} />
      </Route>
    </Routes>
  )
}