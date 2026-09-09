import { useGetSecurityEvents } from '@/api/sea-watch.api'
import { RouterParams } from '@/routes/router'
import { useParams } from 'react-router'
import PageLogsEvents from '../ui/page-logs-events'

export default function PageLogsEventsFeature() {
  const { realm_name } = useParams<RouterParams>()
  const { data, isLoading, isError } = useGetSecurityEvents({ realm: realm_name })

  return <PageLogsEvents events={data?.data ?? []} isLoading={isLoading} isError={isError} />
}
