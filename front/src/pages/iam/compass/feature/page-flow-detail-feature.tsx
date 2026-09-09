import { useNavigate, useParams } from 'react-router'
import { useGetFlow } from '@/api/compass.api'
import { RouterParams } from '@/routes/router'
import { COMPASS_URL } from '@/routes/router'
import PageFlowDetail from '../ui/page-flow-detail'

export default function PageFlowDetailFeature() {
  const { realm_name, flow_id } = useParams<RouterParams & { flow_id: string }>()
  const navigate = useNavigate()
  const realm = realm_name ?? 'master'

  const {
    data: flowResponse,
    isLoading,
    isError,
  } = useGetFlow({ realm, flowId: flow_id ?? '' })

  return (
    <PageFlowDetail
      flow={flowResponse?.data}
      isLoading={isLoading}
      isError={isError}
      onBack={() => navigate(COMPASS_URL(realm))}
    />
  )
}
