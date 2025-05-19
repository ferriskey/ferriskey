import { Route, Routes } from "react-router";
import PageUsersOverviewFeature from './feature/page-users-overview-feature';

export default function PageUser() {
  return (
    <Routes>
      <Route path="overview" element={<PageUsersOverviewFeature />} />
    </Routes>
  )
}