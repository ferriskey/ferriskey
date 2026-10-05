import { useMemo } from 'react'
import { useParams } from 'react-router'
import {
  FLOW_FILTER_KEYS,
  useFlowCount,
  useGetFlows,
  useGetStats,
  type FlowsFilter,
  type FlowsQuery,
} from '@/api/compass.api'
import { RouterParams } from '@/routes/router'
import { Schemas } from '@/api/api.client'
import { usePagedListing } from '@/components/kit'
import { useRealmDirectory } from '@/hooks/use-realm-directory'
import { COMPASS_URL } from '@/routes/router'
import PageFlows from '../ui/page-flows'

import CompassFlow = Schemas.CompassFlow

const FAILURE_SAMPLE = 20

export default function PageFlowsFeature() {
  const { realm_name } = useParams<RouterParams>()
  const realm = realm_name ?? 'master'

  const listing = usePagedListing(FLOW_FILTER_KEYS)

  const {
    data: flowsResponse,
    isLoading: isLoadingFlows,
    isError,
  } = useGetFlows({ realm, query: listing.apiQuery as FlowsQuery, keepPrevious: true })

  const { data: statsResponse, isLoading: isLoadingStats } = useGetStats({ realm })
  const { data: failuresResponse } = useGetFlows({
    realm,
    query: { status: 'failure', limit: FAILURE_SAMPLE },
  })
  const expired = useFlowCount({ realm, filter: { status: 'expired' } })
  const activeFilter = listing.state.filters as FlowsFilter
  const failedInView = useFlowCount({ realm, filter: { ...activeFilter, status: 'failure' } })
  const expiredInView = useFlowCount({ realm, filter: { ...activeFilter, status: 'expired' } })

  const flows = useMemo(() => flowsResponse?.data ?? [], [flowsResponse])
  const userIds = useMemo(() => flows.flatMap((flow) => flow.user_id ?? []), [flows])
  const directory = useRealmDirectory(realm, userIds)

  return (
    <PageFlows
      flows={flows}
      pagination={flowsResponse?.metadata}
      listing={listing}
      stats={statsResponse?.data ?? null}
      counts={{
        failed: failuresResponse?.metadata.total ?? 0,
        expired: expired.count,
        failedInView: failedInView.count,
        expiredInView: expiredInView.count,
      }}
      recentFailures={failuresResponse?.data ?? []}
      isLoading={isLoadingFlows || isLoadingStats}
      isError={isError}
      directory={directory}
      flowHref={(flow: CompassFlow) => `${COMPASS_URL(realm)}/${flow.id}`}
    />
  )
}
