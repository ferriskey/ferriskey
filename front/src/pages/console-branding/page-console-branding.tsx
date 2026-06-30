import { Navigate, Route, Routes } from 'react-router'
import PageEmailTemplateListFeature from '@/pages/email-template/feature/page-email-template-list-feature'
import PageEmailTemplateBuilderFeature from '@/pages/email-template/feature/page-email-template-builder-feature'
import PageThemesListFeature from '@/pages/portal/themes/feature/page-themes-list-feature'
import PageThemeBuilderFeature from '@/pages/portal/themes/feature/page-theme-builder-feature'

export default function PageConsoleBranding() {
  return (
    <Routes>
      <Route index element={<Navigate to='email-templates' replace />} />
      <Route path='email-templates' element={<PageEmailTemplateListFeature />} />
      <Route path='email-templates/:template_id/builder' element={<PageEmailTemplateBuilderFeature />} />
      {/* Portal themes, rendered inside the console chrome (shared with the
          admin portal; navigation is path-aware so it stays in the console). */}
      <Route path='themes' element={<PageThemesListFeature />} />
      <Route path='themes/:theme_id' element={<PageThemeBuilderFeature />} />
      <Route path='themes/:theme_id/:section' element={<PageThemeBuilderFeature />} />
      <Route path='themes/:theme_id/pages/:page_type' element={<PageThemeBuilderFeature />} />
    </Routes>
  )
}
