import { Client } from "@/api/api.interface"
import { useGetClients } from "@/api/client.api"
import { RouterParams } from "@/routes/router"
import { useNavigate, useParams } from "react-router"
import PageClientsOverview from "../ui/page-clients-overview"
import { CLIENT_OVERVIEW_URL } from "@/routes/sub-router/client.router"

export default function PageClientsOverviewFeature() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const { data, isLoading } = useGetClients({ realm: realm_name ?? 'master' })

  const handleDeleteSelected = (items: Client[]) => {
    console.log("Deleting", items);
    // Logique de suppression en lot
  };

  const handleClickRow = (clientId: string) => {
    navigate(`${CLIENT_OVERVIEW_URL(realm_name, clientId)}`)
  }

  return (
    <PageClientsOverview
      data={data?.data || []}
      isLoading={isLoading}
      realmName={realm_name ?? "master"}
      handleDeleteSelected={handleDeleteSelected}
      handleClickRow={handleClickRow}
    />
  )
}
