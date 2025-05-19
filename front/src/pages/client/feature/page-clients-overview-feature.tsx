import { Client } from "@/api/api.interface"
import { useGetClients } from "@/api/client.api"
import { RouterParams } from "@/routes/router"
import { useParams } from "react-router"
import PageClientsOverview from "../ui/page-clients-overview"

export default function PageClientsOverviewFeature() {
  const { realm_name } = useParams<RouterParams>()
  const { data, isLoading } = useGetClients({ realm: 'master' })

  const handleDeleteSelected = (items: Client[]) => {
    console.log("Deleting", items);
    // Logique de suppression en lot
  };

  return (
    <PageClientsOverview 
      data={data?.data || []}
      isLoading={isLoading}
      realmName={realm_name ?? "master"}
      handleDeleteSelected={handleDeleteSelected}
    />
  )
}
