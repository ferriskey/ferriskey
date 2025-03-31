import { Route, Routes } from 'react-router-dom'
import PageLoginFeature from './feature/page-login-feature'
import PageCallbackFeature from './feature/page-callback-feature'

export default function PageAuthentication() {
  return (
    <Routes>
      <Route path='/login' element={<PageLoginFeature />} />
      <Route path='/callback' element={<PageCallbackFeature />} />
    </Routes>
  )
}
