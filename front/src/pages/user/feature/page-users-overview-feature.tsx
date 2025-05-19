import { User } from "@/api/api.interface"
import { RouterParams } from "@/routes/router"
import { useParams } from "react-router"
import { useGetUsers } from '../../../api/user.api'
import PageUsersOverview from '../ui/page-users-overview'

export default function PageUsersOverviewFeature() {
  const { realm_name } = useParams<RouterParams>()
  const { data, isLoading } = useGetUsers({ realm: realm_name ?? 'master' })

  const handleDeleteSelected = (items: User[]) => {
    console.log("Deleting", items);
    // Logique de suppression en lot
  };

  return (
    <PageUsersOverview
      data={data?.data || []}
      isLoading={isLoading}
      realmName={realm_name ?? "master"}
      handleDeleteSelected={handleDeleteSelected}
    />
  )
}
