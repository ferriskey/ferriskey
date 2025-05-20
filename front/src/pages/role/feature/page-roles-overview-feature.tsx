import { Role } from "@/api/api.interface"
import { RouterParams } from "@/routes/router"
import { useParams } from "react-router"
import { useGetRoles } from '../../../api/role.api'
import PageRolesOverview from '../ui/page-roles-overview'

export default function PageRolesOverviewFeature() {
  const { realm_name } = useParams<RouterParams>()
  const { data, isLoading } = useGetRoles({ realm: realm_name ?? 'master' })

  const handleDeleteSelected = (items: Role[]) => {
    console.log("Deleting", items);
  };

  return (
    <PageRolesOverview
      data={data?.data || []}
      isLoading={isLoading}
      realmName={realm_name ?? "master"}
      handleDeleteSelected={handleDeleteSelected}
    />
  )
}
