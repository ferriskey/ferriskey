import { User } from "@/api/api.interface"
import { RouterParams } from "@/routes/router"
import { useParams } from "react-router"
import { toast } from 'sonner'
import { useBulkDeleteUser, useGetUsers } from '../../../api/user.api'
import PageUsersOverview from '../ui/page-users-overview'

export default function PageUsersOverviewFeature() {
  const { realm_name } = useParams<RouterParams>()
  const { data, isLoading } = useGetUsers({ realm: realm_name ?? 'master' })
  const { mutate: bulkDeleteUser } = useBulkDeleteUser()

  const handleDeleteSelected = (items: User[]) => {
    bulkDeleteUser({
      realm: realm_name ?? 'master',
      payload: { ids: items.map((item) => item.id) }
    }, {
      onSuccess: () => {
        toast.success(`${items.length} users deleted`)
      },
      onError: (error) => {
        toast.error(error.message)
      }
    })
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
