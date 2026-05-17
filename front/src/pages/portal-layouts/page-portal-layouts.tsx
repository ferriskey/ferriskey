import { Route, Routes } from 'react-router-dom'
import PagePortalLayoutBuilderFeature from './feature/page-portal-layout-builder-feature'
import PagePortalLayoutsListFeature from './feature/page-portal-layouts-list-feature'

export default function PagePortalLayouts() {
  return (
    <Routes>
      <Route index element={<PagePortalLayoutsListFeature />} />
      <Route path='/:layout_id' element={<PagePortalLayoutBuilderFeature />} />
    </Routes>
  )
}
