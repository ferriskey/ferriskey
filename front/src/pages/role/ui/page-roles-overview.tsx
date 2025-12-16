import { DataTable } from '@/components/ui/data-table'
import { Edit, ExternalLink, Trash2, Shield, ShieldCheck, ShieldAlert, Activity } from 'lucide-react'
import { columns } from '../columns/list-client.column'
import { Schemas } from '@/api/api.client'
import { ConfirmDeleteAlert } from '@/components/confirm-delete-alert'
import { Filter, FilterFieldsConfig } from '@/components/ui/filters'
import StatisticsCard from '../components/statistics-card'

import Role = Schemas.Role

interface Statistics {
  totalRoles: number
  realmRoles: number
  clientRoles: number
  rolesWithPermissions: number
}

interface ConfirmState {
  title: string
  description: string
  open: boolean
  onConfirm: () => void
}

export interface PageRolesOverviewProps {
  isLoading?: boolean
  data: Role[]
  realmName: string
  statistics: Statistics
  filters: Filter[]
  filterFields: FilterFieldsConfig
  onFiltersChange: (filters: Filter[]) => void
  confirm: ConfirmState
  onConfirmClose: () => void
  handleDeleteSelected: (items: Role[]) => void
  handleClickRow: (roleId: string) => void
  handleCreateRole: () => void
  onRowDelete: (role: Role) => void
}

export default function PageRolesOverview({
  data,
  isLoading,
  statistics,
  filters,
  filterFields,
  onFiltersChange,
  confirm,
  onConfirmClose,
  handleDeleteSelected,
  handleClickRow,
  handleCreateRole,
  onRowDelete,
}: PageRolesOverviewProps) {
  const { totalRoles, realmRoles, clientRoles, rolesWithPermissions } = statistics

  return (
    <div className='flex flex-col gap-6'>
      {/* Statistics Cards */}
      <div className='grid gap-4 md:grid-cols-2 lg:grid-cols-4'>
        <StatisticsCard
          title='Total Roles'
          value={totalRoles}
          description='All registered roles'
          icon={Shield}
          isLoading={isLoading}
        />

        <StatisticsCard
          title='Realm Roles'
          value={realmRoles}
          description={
            realmRoles > 0 && totalRoles > 0 ? (
              <span className='text-blue-600 font-medium'>
                {((realmRoles / totalRoles) * 100).toFixed(0)}% of total
              </span>
            ) : (
              'No realm roles'
            )
          }
          icon={ShieldCheck}
          isLoading={isLoading}
        />

        <StatisticsCard
          title='Client Roles'
          value={clientRoles}
          description='Client-specific roles'
          icon={ShieldAlert}
          isLoading={isLoading}
        />

        <StatisticsCard
          title='With Permissions'
          value={rolesWithPermissions}
          description='Roles with permissions'
          icon={Activity}
          isLoading={isLoading}
        />
      </div>

      {/* Data Table */}
      <DataTable
        data={data}
        columns={columns}
        isLoading={isLoading}
        searchPlaceholder='Search roles by name or description...'
        createData={{
          label: 'Create Role',
          onClick: handleCreateRole,
        }}
        searchKeys={['name', 'description']}
        enableSelection={true}
        enableFilters={true}
        filterFields={filterFields}
        filters={filters}
        onFiltersChange={onFiltersChange}
        onRowClick={(role) => {
          handleClickRow(role.id)
        }}
        onDeleteSelected={handleDeleteSelected}
        rowActions={[
          {
            label: 'Edit',
            icon: <Edit className='h-4 w-4' />,
            onClick: (role) => console.log('Edit', role),
          },
          {
            label: 'View',
            icon: <ExternalLink className='h-4 w-4' />,
            onClick: (role) => console.log('View', role),
          },
          {
            label: 'Delete',
            icon: <Trash2 className='h-4 w-4' />,
            variant: 'destructive',
            onClick: (role) => onRowDelete(role),
          },
        ]}
      />
      <ConfirmDeleteAlert
        title={confirm.title}
        description={confirm.description}
        open={confirm.open}
        onConfirm={confirm.onConfirm}
        onCancel={onConfirmClose}
      />
    </div>
  )
}
