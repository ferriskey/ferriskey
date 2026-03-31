import { Route, Routes } from 'react-router'
import OrganizationLayout from './layout/organization-layout'
import PageOrganizationsOverviewFeature from './feature/page-organizations-overview-feature'
import PageOrganizationSettingsFeature from './feature/page-organization-settings-feature'
import PageOrganizationAttributesFeature from './feature/page-organization-attributes-feature'
import PageCreateOrganizationFeature from './feature/page-create-organization-feature'

export default function PageOrganization() {
  return (
    <Routes>
      <Route path='/overview' element={<PageOrganizationsOverviewFeature />} />
      <Route path='/create' element={<PageCreateOrganizationFeature />} />

      <Route path='/:organizationId' element={<OrganizationLayout />}>
        <Route path='settings' element={<PageOrganizationSettingsFeature />} />
        <Route path='attributes' element={<PageOrganizationAttributesFeature />} />
      </Route>
    </Routes>
  )
}
