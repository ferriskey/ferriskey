import { useParams } from 'react-router'
import { RouterParams } from '@/routes/router'
import { useListingQuery } from '@/components/kit'
import { useRealmDirectory } from '@/next/shared/use-realm-directory'
import { eventFamilies } from '@/next/pages/iam/seawatch/event-catalogue'
import { useWindowEvents } from './use-window-events'
import PageLogs from '../ui/page-logs'

export default function PageLogsFeature() {
  const { realm_name } = useParams<RouterParams>()
  const realm = realm_name ?? 'master'

  const listing = useListingQuery()
  const directory = useRealmDirectory(realm)

  const { events, isLoading, isError, truncated, windowDays, windowLimit } = useWindowEvents(
    realm,
    eventFamilies[listing.filter]
  )

  return (
    <PageLogs
      events={events}
      isLoading={isLoading}
      isError={isError}
      truncated={truncated}
      windowDays={windowDays}
      windowLimit={windowLimit}
      listing={listing}
      directory={directory}
    />
  )
}
