import { useGetSecurityEvents } from '@/api/sea-watch.api'
import { RouterParams } from '@/routes/router'
import { useParams } from 'react-router'
import PageOverview from '../ui/page-overview'

export default function PageOverviewFeature() {
  const { realm_name } = useParams<RouterParams>()
  const {
    data: responseGetSecurityEvents,
    isLoading,
    isError,
  } = useGetSecurityEvents({ realm: realm_name })

  return (
    <PageOverview
      events={responseGetSecurityEvents?.data ?? []}
      isLoading={isLoading}
      isError={isError}
      realmName={realm_name}
    />
  )
}
