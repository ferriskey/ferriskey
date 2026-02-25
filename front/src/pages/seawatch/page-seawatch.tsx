import { Route, Routes } from 'react-router-dom'
import PageOverviewFeature from './feature/page-overview-feature'

export default function PageSeawatch() {
  return (
    <Routes>
      <Route path='/overview' element={<PageOverviewFeature />} />
    </Routes>
  )
}
