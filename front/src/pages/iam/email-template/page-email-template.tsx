import { Navigate, Route, Routes } from 'react-router'
import PageEmailsFeature from './feature/page-emails-feature'
import PageEmailTemplateDetailFeature from './feature/page-email-template-detail-feature'
import PageEmailTemplateBuilderFeature from './feature/page-email-template-builder-feature'

export default function PageEmailTemplates() {
  return (
    <Routes>
      <Route index element={<Navigate to='templates' replace />} />
      <Route path='templates' element={<PageEmailsFeature />} />
      <Route path='smtp' element={<PageEmailsFeature />} />
      <Route path='create/builder' element={<PageEmailTemplateBuilderFeature />} />
      <Route path=':template_id/builder' element={<PageEmailTemplateBuilderFeature />} />
      <Route path=':template_id' element={<PageEmailTemplateDetailFeature />} />
    </Routes>
  )
}
