import { Route, Routes } from "react-router";
import PageRolesOverviewFeature from './feature/page-roles-overview-feature';

export default function PageRole() {
  return (
    <Routes>
      <Route path="overview" element={<PageRolesOverviewFeature />} />
    </Routes>
  )
}