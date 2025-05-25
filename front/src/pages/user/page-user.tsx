import { Route, Routes } from "react-router";
import PageUsersOverviewFeature from './feature/page-users-overview-feature';
import UserLayout from './layouts/user-layout';
import UsersLayout from './layouts/users-layout';

export default function PageUser() {
  return (
    <Routes>
      <Route element={<UsersLayout />}>
        <Route path="/overview" element={<PageUsersOverviewFeature />} />
      </Route>
      <Route path="/:user_id/*" element={<UserLayout />} />
    </Routes>
  )
}