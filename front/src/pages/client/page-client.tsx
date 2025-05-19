import { Route, Routes } from "react-router";
import PageClientsOverviewFeature from "./feature/page-clients-overview-feature";
import Container from "./container";

export default function PageClient() {
  return (
    <Routes>
      <Route element={<Container />}>
        <Route path="overview" element={<PageClientsOverviewFeature />} />
      </Route>    
    </Routes>
  )
}