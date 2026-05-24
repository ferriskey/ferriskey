import { Navigate, Route, Routes } from 'react-router'
import PageEmailTemplateListFeature from '@/pages/email-template/feature/page-email-template-list-feature'
import PageEmailTemplateBuilderFeature from '@/pages/email-template/feature/page-email-template-builder-feature'

export default function PageConsoleBranding() {
  return (
    <Routes>
      <Route index element={<Navigate to='email-templates' replace />} />
      <Route path='email-templates' element={<PageEmailTemplateListFeature />} />
      <Route path='email-templates/:template_id/builder' element={<PageEmailTemplateBuilderFeature />} />
    </Routes>
  )
}
