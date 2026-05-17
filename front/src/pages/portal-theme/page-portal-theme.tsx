import { Route, Routes } from 'react-router-dom'
import PagePortalBuilderDemo from '../portal-builder-demo/page-portal-builder-demo'
import PagePortalLayouts from '../portal-layouts/page-portal-layouts'
import PagePortalThemeBuilderFeature from './feature/page-portal-theme-builder-feature'

export default function PagePortalTheme() {
  return (
    <Routes>
      <Route path='/theme' element={<PagePortalThemeBuilderFeature />} />
      <Route path='/layouts/*' element={<PagePortalLayouts />} />
      <Route path='/builder-demo' element={<PagePortalBuilderDemo />} />
    </Routes>
  )
}
