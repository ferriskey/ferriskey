import { DataTable } from '@/components/ui/data-table'
import { Edit, ExternalLink, Trash2 } from 'lucide-react'
import { useNavigate } from 'react-router'
import { Fragment } from 'react/jsx-runtime'
import { columns } from '../columns/list-user.column'
import CreateUserModalFeature from '../feature/create-user-modal-feature.tsx'
import { Dispatch, SetStateAction } from 'react'
import { Schemas } from '@/api/api.client.ts'
import { useConfirmDeleteAlert } from '@/hooks/use-confirm-delete-alert.ts'
import { ConfirmDeleteAlert } from '@/components/confirm-delete-alert'

import User = Schemas.User

export interface PageUsersOverviewOverviewProps {
  isLoading?: boolean
  data: User[]
  realmName: string
  handleDeleteSelected: (items: User[]) => void
  handleClickRow: (userId: string) => void
  openCreateUserModal: boolean
  setOpenCreateUserModal: Dispatch<SetStateAction<boolean>>
}

export default function PageUsersOverview({
  isLoading,
  data,
  realmName,
  handleClickRow,
  handleDeleteSelected,
  openCreateUserModal,
  setOpenCreateUserModal,
}: PageUsersOverviewOverviewProps) {
  const navigate = useNavigate()
  const { confirm, ask, close } = useConfirmDeleteAlert()

  const onRowDelete = (user: User) => {
    ask({
      title: 'Delete user?',
      description: `Are you sure you want to delete "${user.username}"?`,
      onConfirm: () => {
        handleDeleteSelected([user])
        close()
      },
    })
  }
  
  return (
    <Fragment>
      <DataTable
        data={data}
        columns={columns}
        isLoading={isLoading}
        searchPlaceholder='Search a user...'
        searchKeys={['username', 'id']}
        enableSelection={true}
        onRowClick={(user) => {
          handleClickRow(user.id)
        }}
        onDeleteSelected={handleDeleteSelected}
        createData={{
          label: 'Create User',
          onClick: () => {
            setOpenCreateUserModal(true)
          },
        }}
        rowActions={[
          {
            label: 'Edit',
            icon: <Edit className='h-4 w-4' />,
            onClick: (user) => navigate(`/realms/${realmName}/users/${user.id}/overview`),
          },
          {
            label: 'View details',
            icon: <ExternalLink className='h-4 w-4' />,
            onClick: (user) => console.log('View details for user:', user.id),
          },
          {
            label: 'Delete',
            icon: <Trash2 className='h-4 w-4' />,
            variant: 'destructive',
            onClick: (user) => onRowDelete(user),
          },
        ]}
      />
      <CreateUserModalFeature
        realm={realmName}
        open={openCreateUserModal}
        setOpen={setOpenCreateUserModal}
      />
      <ConfirmDeleteAlert
        title={confirm.title}
        description={confirm.description}
        open={confirm.open}
        onConfirm={confirm.onConfirm}
        onCancel={close}
      />
    </Fragment>
  )
}
