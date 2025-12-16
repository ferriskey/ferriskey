import { DataTable } from '@/components/ui/data-table'
import { Edit, ExternalLink, Trash2, Users, UserCheck, UserX, Activity } from 'lucide-react'
import { Fragment } from 'react/jsx-runtime'
import { columns } from '../columns/list-user.column'
import CreateUserModalFeature from '../feature/create-user-modal-feature.tsx'
import { Dispatch, SetStateAction } from 'react'
import { Schemas } from '@/api/api.client.ts'
import { ConfirmDeleteAlert } from '@/components/confirm-delete-alert'
import { Filter, FilterFieldsConfig } from '@/components/ui/filters'
import StatisticsCard from '../components/statistics-card'

import User = Schemas.User

interface Statistics {
  totalUsers: number
  enabledUsers: number
  disabledUsers: number
  verifiedUsers: number
}

interface ConfirmState {
  title: string
  description: string
  open: boolean
  onConfirm: () => void
}

export interface PageUsersOverviewOverviewProps {
  isLoading?: boolean
  data: User[]
  realmName: string
  statistics: Statistics
  filters: Filter[]
  filterFields: FilterFieldsConfig
  onFiltersChange: (filters: Filter[]) => void
  confirm: ConfirmState
  onConfirmClose: () => void
  handleDeleteSelected: (items: User[]) => void
  handleClickRow: (userId: string) => void
  openCreateUserModal: boolean
  setOpenCreateUserModal: Dispatch<SetStateAction<boolean>>
  onRowDelete: (user: User) => void
}

export default function PageUsersOverview({
  isLoading,
  data,
  realmName,
  statistics,
  filters,
  filterFields,
  onFiltersChange,
  confirm,
  onConfirmClose,
  handleClickRow,
  handleDeleteSelected,
  openCreateUserModal,
  setOpenCreateUserModal,
  onRowDelete,
}: PageUsersOverviewOverviewProps) {
  const { totalUsers, enabledUsers, disabledUsers, verifiedUsers } = statistics

  return (
    <Fragment>
      <div className='flex flex-col gap-6'>
        {/* Statistics Cards */}
        <div className='grid gap-4 md:grid-cols-2 lg:grid-cols-4'>
          <StatisticsCard
            title='Total Users'
            value={totalUsers}
            description='All registered users'
            icon={Users}
            isLoading={isLoading}
          />

          <StatisticsCard
            title='Enabled Users'
            value={enabledUsers}
            description={
              enabledUsers > 0 && totalUsers > 0 ? (
                <span className='text-emerald-600 font-medium'>
                  {((enabledUsers / totalUsers) * 100).toFixed(0)}% active
                </span>
              ) : (
                'No enabled users'
              )
            }
            icon={UserCheck}
            isLoading={isLoading}
          />

          <StatisticsCard
            title='Disabled Users'
            value={disabledUsers}
            description='Inactive accounts'
            icon={UserX}
            isLoading={isLoading}
          />

          <StatisticsCard
            title='Verified Users'
            value={verifiedUsers}
            description='Email verified accounts'
            icon={Activity}
            isLoading={isLoading}
          />
        </div>

        {/* Data Table */}
        <DataTable
          data={data}
          columns={columns}
          isLoading={isLoading}
          searchPlaceholder='Search users by username or email...'
          searchKeys={['username', 'email', 'id']}
          enableSelection={true}
          enableFilters={true}
          filterFields={filterFields}
          filters={filters}
          onFiltersChange={onFiltersChange}
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
              onClick: (user) => console.log('Edit', user),
            },
            {
              label: 'View',
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
      </div>
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
        onCancel={onConfirmClose}
      />
    </Fragment>
  )
}
