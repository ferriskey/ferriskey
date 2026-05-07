import { useParams } from 'react-router'
import { useGetClients } from '@/api/client.api'
import { useGetUsers } from '@/api/user.api'
import { useGetRoles } from '@/api/role.api'
import { useGetStats } from '@/api/compass.api'
import { useGetRealm } from '@/api/realm.api'
import { RouterParams } from '@/routes/router'
import PageHome from '../ui/page-home'

export default function PageHomeFeature() {
  const { realm_name } = useParams<RouterParams>()

  const { data: clientsData, isLoading: isLoadingClients } = useGetClients({ realm: realm_name })
  const { data: usersData, isLoading: isLoadingUsers } = useGetUsers({ realm: realm_name })
  const { data: rolesData, isLoading: isLoadingRoles } = useGetRoles({ realm: realm_name })
  const { data: flowStats, isLoading: isLoadingFlows } = useGetStats({ realm: realm_name })
  const { data: realm, isLoading: isLoadingRealm } = useGetRealm({ realm: realm_name })

  return (
    <PageHome
      data={{
        clients: clientsData?.data ?? [],
        users: usersData?.data ?? [],
        roles: rolesData?.data ?? [],
        flowStats: flowStats?.data ?? null,
        realmSettings: realm?.settings ?? null,
        isLoading:
          isLoadingClients ||
          isLoadingUsers ||
          isLoadingRoles ||
          isLoadingFlows ||
          isLoadingRealm,
      }}
    />
  )
}
