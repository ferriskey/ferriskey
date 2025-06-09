import { useGetUserCredentials } from "@/api/user.api"
import { RouterParams } from "@/routes/router"
import { useParams } from "react-router"
import PageCredentials from "../ui/page-credentials"
import { useDeleteUserCredential } from '@/api/credential.api.ts'
import { toast } from 'sonner'

export default function PageCredentialFeature() {
  const { realm_name, user_id } = useParams<RouterParams>()

  const { data: credentials } = useGetUserCredentials({
    realm: realm_name ?? '',
    userId: user_id ?? ''
  })

  const { mutate: deleteUserCredentia } = useDeleteUserCredential()


  const handleDeleteUserCredential = (credentialId: string) => {

    deleteUserCredentia({
      realm: realm_name ?? '',
      userId: user_id ?? '',
      credentialId
    })

    toast.success("Credential was deleted")
  }

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
      handleDeleteUserCredential={handleDeleteUserCredential}
    />
  )
}
