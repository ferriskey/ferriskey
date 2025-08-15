import { RouterParams } from "@/routes/router"
import { useNavigate, useParams } from "react-router"
import { useGetRoles } from '../../../api/role.api'
import PageRolesOverview from '../ui/page-roles-overview'
import { ROLE_SETTINGS_URL, ROLE_URL } from "@/routes/sub-router/role.router"
import { Schemas } from '@/api/api.client'
import Role = Schemas.Role

export default function PageRolesOverviewFeature() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const { data: rolesResponse, isLoading } = useGetRoles({ realm: realm_name ?? 'master' })

  const handleDeleteSelected = (items: Role[]) => {
    console.log('Deleting', items)
  }

  const handleClickRow = (roleId: string) => {
    navigate(`${ROLE_URL(realm_name, roleId)}${ROLE_SETTINGS_URL}`)
  }



  return (
    <PageRolesOverview
      data={rolesResponse?.data || []}
      isLoading={isLoading}
      realmName={realm_name ?? 'master'}
      handleDeleteSelected={handleDeleteSelected}
      handleClickRow={handleClickRow}
    />
  )
}
