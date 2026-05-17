import { Route, Routes } from 'react-router-dom'
import PagePortalBuilderDemo from '../portal-builder-demo/page-portal-builder-demo'
import PagePortalThemeBuilderFeature from './feature/page-portal-theme-builder-feature'

export default function PagePortalTheme() {
  return (
    <Routes>
      <Route path='/theme' element={<PagePortalThemeBuilderFeature />} />
      <Route path='/builder-demo' element={<PagePortalBuilderDemo />} />
    </Routes>
  )
}
