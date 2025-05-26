import { useGetClient } from "@/api/client.api"
import { RouterParams } from "@/routes/router"
import { useParams } from "react-router"
import PageClientOverview from "../ui/page-client-overview"

export default function PageClientOverviewFeature() {
  const { realm_name, client_id } = useParams<RouterParams>()
  const { data } = useGetClient({
    realm: realm_name ?? 'master',
    clientId: client_id ?? '',
  })

  return (
    <>
      {data && (
        <PageClientOverview client={data} />
      )}
    </>
  )
}