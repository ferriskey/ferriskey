import { Route, Routes } from 'react-router'
import PageFlowsFeature from './feature/page-flows-feature'
import PageFlowDetailFeature from './feature/page-flow-detail-feature'

export default function NextPageCompass() {
  return (
    <Routes>
      <Route index element={<PageFlowsFeature />} />
      <Route path=':flow_id' element={<PageFlowDetailFeature />} />
    </Routes>
  )
}
