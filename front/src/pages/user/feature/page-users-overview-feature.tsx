import { User } from '@/api/core.interface'
import { RouterParams } from '@/routes/router'
import { useNavigate, useParams } from 'react-router'
import { toast } from 'sonner'
import { useBulkDeleteUser, useGetUsers } from '../../../api/user.api'
import PageUsersOverview from '../ui/page-users-overview'
import { USER_OVERVIEW_URL, USER_URL } from '@/routes/sub-router/user.router'
import { useState } from 'react'

export default function PageUsersOverviewFeature() {
  const { realm_name } = useParams<RouterParams>()
  const { data, isLoading } = useGetUsers({ realm: realm_name ?? 'master' })
  const { mutate: bulkDeleteUser } = useBulkDeleteUser()
  const [openCreateUserModal, setOpenCreateUserModal] = useState(false)
  const navigate = useNavigate()

  const handleDeleteSelected = (items: User[]) => {
    bulkDeleteUser(
      {
        realm: realm_name ?? 'master',
        payload: { ids: items.map((item) => item.id) },
      },
      {
        onSuccess: (data) => toast.success(`${data.count} users deleted`),
        onError: (error) => toast.error(error.message),
      }
    )
  }

  const handleClickRow = (userId: string) => {
    navigate(`${USER_URL(realm_name, userId)}${USER_OVERVIEW_URL}`)
  }

  return (
    <PageUsersOverview
      data={data || []}
      isLoading={isLoading}
      realmName={realm_name ?? 'master'}
      handleDeleteSelected={handleDeleteSelected}
      handleClickRow={handleClickRow}
      openCreateUserModal={openCreateUserModal}
      setOpenCreateUserModal={setOpenCreateUserModal}
    />
  )
}
