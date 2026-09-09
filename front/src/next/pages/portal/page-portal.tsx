import { Navigate, Route, Routes } from 'react-router'
import PagePortalThemesFeature from './feature/page-portal-themes-feature'
import PagePortalThemeDetailFeature from './feature/page-portal-theme-detail-feature'
import PagePortalPageBuilderFeature from './feature/page-portal-page-builder-feature'
import PagePortalLayoutsFeature from './feature/page-portal-layouts-feature'
import PagePortalLayoutBuilderFeature from './feature/page-portal-layout-builder-feature'

export default function NextPagePortal() {
  return (
    <Routes>
      <Route index element={<Navigate to='themes' replace />} />
      <Route path='themes' element={<PagePortalThemesFeature />} />
      <Route
        path='themes/:theme_id/pages/:page_type'
        element={<PagePortalPageBuilderFeature />}
      />
      <Route path='themes/:theme_id' element={<Navigate to='theme' replace />} />
      <Route path='themes/:theme_id/*' element={<PagePortalThemeDetailFeature />} />
      <Route path='layouts' element={<PagePortalLayoutsFeature />} />
      <Route path='layouts/:layout_id' element={<PagePortalLayoutBuilderFeature />} />
    </Routes>
  )
}
