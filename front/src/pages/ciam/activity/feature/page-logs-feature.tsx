import { useMemo } from 'react'
import { useParams } from 'react-router'
import { useTranslation } from 'react-i18next'
import { RouterParams } from '@/routes/router'
import { useListingQuery } from '@/components/kit'
import { eventUserIds, useRealmDirectory } from '@/hooks/use-realm-directory'
import { eventRoleIds } from '@/hooks/event-role-ids'
import { eventFamilies } from '@/pages/iam/seawatch/event-catalogue'
import { useWindowEvents } from './use-window-events'
import PageLogs from '../ui/page-logs'

const SEAWATCH_NAMESPACE = 'seawatch'

export default function PageLogsFeature() {
  useTranslation(SEAWATCH_NAMESPACE)

  const { realm_name } = useParams<RouterParams>()
  const realm = realm_name ?? 'master'

  const listing = useListingQuery()
  const { events, isLoading, isError, truncated, windowDays, windowLimit } = useWindowEvents(
    realm,
    eventFamilies[listing.filter]
  )
  const userIds = useMemo(() => eventUserIds(events), [events])
  const roleIds = useMemo(() => eventRoleIds(events), [events])
  const directory = useRealmDirectory(realm, userIds, roleIds)

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
