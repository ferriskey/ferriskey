import { Route, Routes } from 'react-router'
import PageFlowsFeature from './feature/page-flows-feature'
import PageFlowDetailFeature from './feature/page-flow-detail-feature'

export default function PageCompass() {
  return (
    <Routes>
      <Route path='/overview' element={<PageFlowsFeature />} />
      <Route path='/:flow_id' element={<PageFlowDetailFeature />} />
    </Routes>
  )
}
