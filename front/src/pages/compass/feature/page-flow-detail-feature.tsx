import { useGetFlow } from '@/api/compass.api'
import { RouterParams } from '@/routes/router'
import { useParams } from 'react-router'
import PageFlowDetail from '../ui/page-flow-detail'

export default function PageFlowDetailFeature() {
  const { realm_name, flow_id } = useParams<RouterParams & { flow_id: string }>()

  const {
    data: responseGetFlow,
    isLoading,
    isError,
  } = useGetFlow({ realm: realm_name, flowId: flow_id! })

  return (
    <PageFlowDetail
      flow={responseGetFlow?.data ?? null}
      isLoading={isLoading}
      isError={isError}
    />
  )
}
