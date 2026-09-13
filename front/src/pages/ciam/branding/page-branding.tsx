import { Navigate, Route, Routes } from 'react-router'
import PageEmailTemplates from '@/pages/iam/email-template/page-email-template'
import PagePortalThemesFeature from '@/pages/iam/portal/feature/page-portal-themes-feature'
import PagePortalThemeDetailFeature from '@/pages/iam/portal/feature/page-portal-theme-detail-feature'
import PagePortalPageBuilderFeature from '@/pages/iam/portal/feature/page-portal-page-builder-feature'
import PagePortalLayoutsFeature from '@/pages/iam/portal/feature/page-portal-layouts-feature'
import PagePortalLayoutBuilderFeature from '@/pages/iam/portal/feature/page-portal-layout-builder-feature'

export default function ConsoleBranding() {
  return (
    <Routes>
      <Route index element={<Navigate to='email-templates' replace />} />
      <Route path='email-templates/*' element={<PageEmailTemplates />} />
      <Route path='themes' element={<PagePortalThemesFeature />} />
      <Route path='themes/:theme_id/pages/:page_type' element={<PagePortalPageBuilderFeature />} />
      <Route path='themes/:theme_id' element={<Navigate to='theme' replace />} />
      <Route path='themes/:theme_id/*' element={<PagePortalThemeDetailFeature />} />
      <Route path='layouts' element={<PagePortalLayoutsFeature />} />
      <Route path='layouts/:layout_id' element={<PagePortalLayoutBuilderFeature />} />
    </Routes>
  )
}
