import { useGetUserCredentials } from "@/api/user.api"
import { RouterParams } from "@/routes/router"
import { useParams } from "react-router"
import PageCredentials from "../ui/page-credentials"

export default function PageCredentialFeature() {
  const { realm_name, user_id } = useParams<RouterParams>()

  const { data: credentials } = useGetUserCredentials({
    realm: realm_name ?? '',
    userId: user_id ?? ''
  })

  if (!credentials) {
    return (
      <div className="p-6">
        <h1 className="text-2xl font-bold tracking-tight">Credentials</h1>
        <p className="text-muted-foreground">Loading...</p>
      </div>
    )
  }
  return (
    <PageCredentials 
      credentials={credentials}
    />
  )
}