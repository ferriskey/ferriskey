import { Route, Routes } from 'react-router'
import PageAccountFeature from './feature/page-account-feature'

export default function PageAccount() {
  return (
    <Routes>
      <Route index element={<PageAccountFeature />} />
    </Routes>
  )
}
