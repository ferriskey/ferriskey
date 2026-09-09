import { useMemo } from 'react'
import { useParams } from 'react-router'
import { useGetFlows, useGetStats } from '@/api/compass.api'
import { RouterParams } from '@/routes/router'
import { Schemas } from '@/api/api.client'
import { NEXT_COMPASS_URL } from '@/next/routes'
import PageFlows from '../ui/page-flows'

import CompassFlow = Schemas.CompassFlow

export default function PageFlowsFeature() {
  const { realm_name } = useParams<RouterParams>()
  const realm = realm_name ?? 'master'

  const {
    data: flowsResponse,
    isLoading: isLoadingFlows,
    isError,
  } = useGetFlows({ realm })

  const { data: statsResponse, isLoading: isLoadingStats } = useGetStats({ realm })

  const flows = useMemo(() => flowsResponse?.data ?? [], [flowsResponse])

  return (
    <PageFlows
      flows={flows}
      stats={statsResponse?.data ?? null}
      isLoading={isLoadingFlows || isLoadingStats}
      isError={isError}
      flowHref={(flow: CompassFlow) => `${NEXT_COMPASS_URL(realm)}/${flow.id}`}
    />
  )
}
