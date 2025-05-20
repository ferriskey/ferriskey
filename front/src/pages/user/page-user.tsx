import { Route, Routes } from "react-router";
import PageUsersOverviewFeature from './feature/page-users-overview-feature';
import Container from "./container";

export default function PageUser() {
  return (
    <Routes>
      <Route element={<Container />}>
      <Route path="overview" element={<PageUsersOverviewFeature />} />
      </Route>
      
    </Routes>
  )
}