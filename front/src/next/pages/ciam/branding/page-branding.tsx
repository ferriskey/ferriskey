import { Navigate, Route, Routes } from 'react-router'
import NextPageEmailTemplates from '@/next/pages/iam/email-template/page-email-template'
import PagePortalThemesFeature from '@/next/pages/iam/portal/feature/page-portal-themes-feature'
import PagePortalThemeDetailFeature from '@/next/pages/iam/portal/feature/page-portal-theme-detail-feature'
import PagePortalPageBuilderFeature from '@/next/pages/iam/portal/feature/page-portal-page-builder-feature'
import PagePortalLayoutsFeature from '@/next/pages/iam/portal/feature/page-portal-layouts-feature'
import PagePortalLayoutBuilderFeature from '@/next/pages/iam/portal/feature/page-portal-layout-builder-feature'

export default function ConsoleBranding() {
  return (
    <Routes>
      <Route index element={<Navigate to='email-templates' replace />} />
      <Route path='email-templates/*' element={<NextPageEmailTemplates />} />
      <Route path='themes' element={<PagePortalThemesFeature />} />
      <Route path='themes/:theme_id/pages/:page_type' element={<PagePortalPageBuilderFeature />} />
      <Route path='themes/:theme_id' element={<Navigate to='theme' replace />} />
      <Route path='themes/:theme_id/*' element={<PagePortalThemeDetailFeature />} />
      <Route path='layouts' element={<PagePortalLayoutsFeature />} />
      <Route path='layouts/:layout_id' element={<PagePortalLayoutBuilderFeature />} />
    </Routes>
  )
}
