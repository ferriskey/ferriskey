import { useGetFlows, useGetStats } from '@/api/compass.api'
import { RouterParams } from '@/routes/router'
import { useParams } from 'react-router'
import PageFlows from '../ui/page-flows'

export default function PageFlowsFeature() {
  const { realm_name } = useParams<RouterParams>()
  const {
    data: responseGetFlows,
    isLoading: isLoadingFlows,
    isError: isErrorFlows,
  } = useGetFlows({ realm: realm_name })

  const {
    data: responseGetStats,
    isLoading: isLoadingStats,
  } = useGetStats({ realm: realm_name })

  return (
    <PageFlows
      flows={responseGetFlows?.data ?? []}
      stats={responseGetStats?.data ?? null}
      isLoading={isLoadingFlows || isLoadingStats}
      isError={isErrorFlows}
      realmName={realm_name}
    />
  )
}
