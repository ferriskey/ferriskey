import { Route, Routes } from 'react-router-dom'
import PageEmailTemplateListFeature from './feature/page-email-template-list-feature'
import PageEmailTemplateBuilderFeature from './feature/page-email-template-builder-feature'

export default function PageEmailTemplate() {
  return (
    <Routes>
      <Route path='/overview' element={<PageEmailTemplateListFeature />} />
      <Route path='/:template_id/builder' element={<PageEmailTemplateBuilderFeature />} />
    </Routes>
  )
}
