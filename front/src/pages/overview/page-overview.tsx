import { Route, Routes } from 'react-router'
import PageHomeFeature from './feature/page-home-feature'

export default function PageOverview() {
  return (
    <Routes>
      <Route index element={<PageHomeFeature />} />
    </Routes>
  )
}
