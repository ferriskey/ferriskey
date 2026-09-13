import { useMemo } from 'react'
import { useParams } from 'react-router'
import { RouterParams } from '@/routes/router'
import { useGetDailyActivityStats, useGetFlows } from '@/api/compass.api'
import { useGetRealm } from '@/api/realm.api'
import { useRealmDirectory } from '@/hooks/use-realm-directory'
import PageLive from '../ui/page-live'

const FETCH_DAYS = 90
const FLOW_COUNT = 10

const toDateParam = (date: Date) => date.toISOString().slice(0, 10)

export default function PageLiveFeature() {
  const { realm_name } = useParams<RouterParams>()
  const realm = realm_name ?? 'master'

  const { from, to } = useMemo(() => {
    const toDate = new Date()
    const fromDate = new Date(toDate)
    fromDate.setDate(fromDate.getDate() - (FETCH_DAYS - 1))

    return { from: toDateParam(fromDate), to: toDateParam(toDate) }
  }, [])

  const { data: realmResponse } = useGetRealm({ realm })
  const compassEnabled = Boolean(realmResponse?.settings?.compass_enabled)
  const compassRealm = compassEnabled ? realm : undefined

  const {
    data: activityResponse,
    isLoading: isLoadingActivity,
    isError,
  } = useGetDailyActivityStats({ realm: compassRealm, from, to })

  const { data: flowsResponse, isLoading: isLoadingFlows } = useGetFlows({
    realm: compassRealm,
    limit: FLOW_COUNT,
  })

  const directory = useRealmDirectory(realm)

  const activity = useMemo(() => activityResponse?.data ?? [], [activityResponse])
  const flows = useMemo(() => flowsResponse?.data ?? [], [flowsResponse])

  return (
    <PageLive
      activity={activity}
      flows={flows}
      compassEnabled={compassEnabled}
      isLoading={isLoadingActivity || isLoadingFlows}
      isError={isError}
      directory={directory}
    />
  )
}
