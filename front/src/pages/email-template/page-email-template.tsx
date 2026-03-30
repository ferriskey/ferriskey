import { Route, Routes } from 'react-router-dom'
import PageEmailTemplateBuilderFeature from './feature/page-email-template-builder-feature'

export default function PageEmailTemplate() {
  return (
    <Routes>
      <Route path='/:template_id/builder' element={<PageEmailTemplateBuilderFeature />} />
    </Routes>
  )
}
