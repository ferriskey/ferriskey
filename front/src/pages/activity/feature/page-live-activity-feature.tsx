import { useGetFlows } from '@/api/compass.api'
import { useGetUsers } from '@/api/user.api'
import { RouterParams } from '@/routes/router'
import { useParams } from 'react-router'
import PageLiveActivity from '../ui/page-live-activity'

export default function PageLiveActivityFeature() {
  const { realm_name } = useParams<RouterParams>()
  const { data: usersData, isLoading: isLoadingUsers } = useGetUsers({ realm: realm_name })
  const { data: flowsData, isLoading: isLoadingFlows } = useGetFlows({
    realm: realm_name,
    status: 'success',
  })

  return (
    <PageLiveActivity
      users={usersData?.data ?? []}
      flows={flowsData?.data ?? []}
      isLoading={isLoadingUsers || isLoadingFlows}
    />
  )
}
