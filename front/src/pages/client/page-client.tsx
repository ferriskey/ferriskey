import { Route, Routes } from "react-router";
import PageClientsOverviewFeature from "./feature/page-clients-overview-feature";

export default function PageClient() {
  return (
    <Routes>
      <Route path="overview" element={<PageClientsOverviewFeature />} />
    </Routes>
  )
}