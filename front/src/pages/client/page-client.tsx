import { Route, Routes } from "react-router";
import PageClientsOverviewFeature from "./feature/page-clients-overview-feature";
import Container from "./container";
import PageClientOverviewFeature from "./feature/page-client-overview-feature";

export default function PageClient() {
  return (
    <Routes>
      <Route element={<Container />}>
        <Route path="/overview" element={<PageClientsOverviewFeature />} />

        <Route path="/:client_id/overview" element={<PageClientOverviewFeature />} />
      </Route>    
    </Routes>
  )
}