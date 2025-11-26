import { useParams } from 'react-router'
import { useGetClients } from '@/api/client.api'
import { useGetUsers } from '@/api/user.api'
import { useGetRoles } from '@/api/role.api'
import { RouterParams } from '@/routes/router'
import PageHome from '../ui/page-home'

export interface PageHomeData {
  clients: any[]
  users: any[]
  roles: any[]
  isLoading: boolean
}

export default function PageHomeFeature() {
  const { realm_name } = useParams<RouterParams>()

  // Fetch data using hooks
  const { data: clientsData, isLoading: isLoadingClients } = useGetClients({ realm: realm_name })
  const { data: usersData, isLoading: isLoadingUsers } = useGetUsers({ realm: realm_name })
  const { data: rolesData, isLoading: isLoadingRoles } = useGetRoles({ realm: realm_name })

  // Prepare data for UI component
  const homeData: PageHomeData = {
    clients: clientsData?.data || [],
    users: usersData?.data || [],
    roles: rolesData?.data || [],
    isLoading: isLoadingClients || isLoadingUsers || isLoadingRoles,
  }

  return <PageHome data={homeData} />
}
