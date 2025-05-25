import { useParams } from 'react-router';
import { useGetUser } from '../../../api/user.api';
import { UserRouterParams } from '../../../routes/sub-router/user.router';
import EditUserOverview from '../ui/edit-user-overview';

export default function EditUserOverviewFeature() {
  const { realm_name, user_id, current_view } = useParams<UserRouterParams>()
  const { data, isLoading } = useGetUser({ realm: realm_name, userId: user_id })

  if (!user_id || isLoading) return

  return (
    <EditUserOverview
      realm={realm_name ?? 'master'}
      currentView={current_view ?? 'overview'}
      user={data}
    />
  )
}