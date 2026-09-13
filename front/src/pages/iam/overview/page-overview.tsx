import { Route, Routes } from 'react-router'
import PageOverviewFeature from './feature/page-overview-feature'

export default function PageOverview() {
  return (
    <Routes>
      <Route index element={<PageOverviewFeature />} />
    </Routes>
  )
}
