import { useMemo } from 'react'
import { useParams } from 'react-router'
import { useGetFlows, useGetStats } from '@/api/compass.api'
import { RouterParams } from '@/routes/router'
import { Schemas } from '@/api/api.client'
import { useListingQuery } from '@/components/kit'
import { useRealmDirectory } from '@/hooks/use-realm-directory'
import { COMPASS_URL } from '@/routes/router'
import PageFlows from '../ui/page-flows'

import CompassFlow = Schemas.CompassFlow

export default function PageFlowsFeature() {
  const { realm_name } = useParams<RouterParams>()
  const realm = realm_name ?? 'master'

  const listing = useListingQuery()
  const directory = useRealmDirectory(realm)

  const {
    data: flowsResponse,
    isLoading: isLoadingFlows,
    isError,
  } = useGetFlows({
    realm,
    status: listing.filter === 'failed' ? 'failure' : undefined,
  })

  const { data: statsResponse, isLoading: isLoadingStats } = useGetStats({ realm })

  const flows = useMemo(() => flowsResponse?.data ?? [], [flowsResponse])

  return (
    <PageFlows
      flows={flows}
      stats={statsResponse?.data ?? null}
      isLoading={isLoadingFlows || isLoadingStats}
      isError={isError}
      listing={listing}
      directory={directory}
      flowHref={(flow: CompassFlow) => `${COMPASS_URL(realm)}/${flow.id}`}
    />
  )
}
