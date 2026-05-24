import { Route, Routes } from 'react-router-dom'
import PagePortalBuilderDemo from '../portal-builder-demo/page-portal-builder-demo'
import PagePortalLayoutBuilderFeature from '../portal-layouts/feature/page-portal-layout-builder-feature'
import PagePortalLayoutsListFeature from '../portal-layouts/feature/page-portal-layouts-list-feature'
import PagePortalThemeBuilderFeature from '../portal-theme/feature/page-portal-theme-builder-feature'
import PageThemeBuilderFeature from './themes/feature/page-theme-builder-feature'
import PageThemesListFeature from './themes/feature/page-themes-list-feature'

export default function PagePortal() {
  return (
    <Routes>
      {/* Full-viewport builders — no shared chrome. Every tab is addressable
          so it can be shared via link or restored after a refresh. */}
      <Route path='/themes/:theme_id' element={<PageThemeBuilderFeature />} />
      <Route path='/themes/:theme_id/:section' element={<PageThemeBuilderFeature />} />
      <Route
        path='/themes/:theme_id/pages/:page_type'
        element={<PageThemeBuilderFeature />}
      />
      <Route path='/layouts/:layout_id' element={<PagePortalLayoutBuilderFeature />} />

      {/* Legacy single-theme editor + sandbox demo (kept until cleanup PR). */}
      <Route path='/theme' element={<PagePortalThemeBuilderFeature />} />
      <Route path='/builder-demo' element={<PagePortalBuilderDemo />} />

      {/* List pages render their own OverviewHeader (with Themes | Layouts
          tabs) so the chrome matches the rest of the admin console. */}
      <Route index element={<PageThemesListFeature />} />
      <Route path='/themes' element={<PageThemesListFeature />} />
      <Route path='/layouts' element={<PagePortalLayoutsListFeature />} />
    </Routes>
  )
}
