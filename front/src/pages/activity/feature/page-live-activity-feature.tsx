import { useGetDailyActivityStats } from '@/api/compass.api'
import { RouterParams } from '@/routes/router'
import { useMemo } from 'react'
import { useParams } from 'react-router'
import PageLiveActivity from '../ui/page-live-activity'

function toDateParam(date: Date) {
  return date.toISOString().slice(0, 10)
}

export default function PageLiveActivityFeature() {
  const { realm_name } = useParams<RouterParams>()
  const { from, to } = useMemo(() => {
    const toDate = new Date()
    const fromDate = new Date(toDate)
    fromDate.setDate(fromDate.getDate() - 89)

    return {
      from: toDateParam(fromDate),
      to: toDateParam(toDate),
    }
  }, [])

  const { data: activityData, isLoading } = useGetDailyActivityStats({
    realm: realm_name,
    from,
    to,
  })

  return (
    <PageLiveActivity
      dailyActivity={activityData?.data ?? []}
      isLoading={isLoading}
    />
  )
}
