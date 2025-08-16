import { Route, Routes } from "react-router";
import PageRealmSettingsFeature from "./feature/page-realm-settings-feature";

export default function PageRealm() {
  return (
    <Routes>
      <Route index element={<PageRealmSettingsFeature />} />
    </Routes>
  )
}
