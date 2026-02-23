import { RouterParams } from '@/routes/router'
import { useNavigate, useParams } from 'react-router'
import { toast } from 'sonner'
import { useBulkDeleteUser, useGetUsers } from '../../../api/user.api'
import PageUsersOverview from '../ui/page-users-overview'
import { USER_OVERVIEW_URL, USER_URL } from '@/routes/sub-router/user.router'
import { useMemo, useState, useEffect } from 'react'
import { Schemas } from '@/api/api.client.ts'
import { Filter, FilterFieldsConfig } from '@/components/ui/filters'
import { useConfirmDeleteAlert } from '@/hooks/use-confirm-delete-alert'
import { useOutletContext } from 'react-router'
import { UsersLayoutContext } from '../layouts/users-layout'

import User = Schemas.User

export default function PageUsersOverviewFeature() {
  const { realm_name } = useParams<RouterParams>()
  const { data: responseGetUsers, isLoading } = useGetUsers({ realm: realm_name ?? 'master' })
  const { mutate: bulkDeleteUser } = useBulkDeleteUser()
  const [openCreateUserModal, setOpenCreateUserModal] = useState(false)
  const { confirm, ask, close } = useConfirmDeleteAlert()
  const { setPrimaryAction } = useOutletContext<UsersLayoutContext>()

  useEffect(() => {
    setPrimaryAction({ label: 'New User', onClick: () => setOpenCreateUserModal(true) })
    return () => setPrimaryAction(undefined)
  }, [])
  const [filters, setFilters] = useState<Filter[]>([])
  const navigate = useNavigate()

  const users = useMemo(() => responseGetUsers?.data || [], [responseGetUsers])

  // Calculate statistics
  const statistics = useMemo(() => {
    const totalUsers = users.length
    const enabledUsers = users.filter(user => user.enabled).length
    const disabledUsers = users.filter(user => !user.enabled).length
    const verifiedUsers = users.filter(user => user.email_verified).length

    return {
      totalUsers,
      enabledUsers,
      disabledUsers,
      verifiedUsers,
    }
  }, [users])

  // Filter configuration
  const filterFields: FilterFieldsConfig = []

  const handleDeleteSelected = (items: User[]) => {
    if (!realm_name) return
    bulkDeleteUser(
      {
        path: {
          realm_name
        },
        body: {
          ids: items.map(i => i.id)
        }
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

  const onRowDelete = (user: User) => {
    ask({
      title: 'Delete user?',
      description: `Are you sure you want to delete "${user.username}"? This action cannot be undone.`,
      onConfirm: () => {
        handleDeleteSelected([user])
        close()
      },
    })
  }

  const handleFiltersChange = (newFilters: Filter[]) => {
    setFilters(newFilters)
  }

  return (
    <PageUsersOverview
      data={users}
      isLoading={isLoading}
      realmName={realm_name ?? 'master'}
      statistics={statistics}
      filters={filters}
      filterFields={filterFields}
      onFiltersChange={handleFiltersChange}
      confirm={confirm}
      onConfirmClose={close}
      handleDeleteSelected={handleDeleteSelected}
      handleClickRow={handleClickRow}
      openCreateUserModal={openCreateUserModal}
      setOpenCreateUserModal={setOpenCreateUserModal}
      onRowDelete={onRowDelete}
    />
  )
}
