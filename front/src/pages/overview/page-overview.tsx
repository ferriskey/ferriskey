import { Route, Routes } from 'react-router-dom'
import PageHomeFeature from './feature/page-home-feature'

export default function PageOverview() {
  return (
    <Routes>
      <Route index element={<PageHomeFeature />} />
    </Routes>
  )
}
