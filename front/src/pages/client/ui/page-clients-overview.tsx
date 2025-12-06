import { DataTable } from '@/components/ui/data-table'
import { Edit, ExternalLink, Trash2, Users, Globe, Lock, Activity } from 'lucide-react'
import { columns } from '../columns/list-client.column'
import { Schemas } from '@/api/api.client.ts'
import { ConfirmDeleteAlert } from '@/components/confirm-delete-alert'
import { Filter, FilterFieldsConfig } from '@/components/ui/filters'
import StatisticsCard from '../components/statistics-card'

import Client = Schemas.Client

interface Statistics {
  totalClients: number
  activeClients: number
  publicClients: number
  confidentialClients: number
}

interface ConfirmState {
  title: string
  description: string
  open: boolean
  onConfirm: () => void
}

export interface PageClientsOverviewProps {
  isLoading?: boolean
  data: Client[]
  realmName: string
  statistics: Statistics
  filters: Filter[]
  filterFields: FilterFieldsConfig
  onFiltersChange: (filters: Filter[]) => void
  confirm: ConfirmState
  onConfirmClose: () => void
  handleDeleteSelected: (items: Client[]) => void
  handleClickRow: (clientId: string) => void
  handleCreateClient: () => void
  onRowDelete: (client: Client) => void
}

export default function PageClientsOverview({
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
  handleCreateClient,
  onRowDelete,
}: PageClientsOverviewProps) {
  const { totalClients, activeClients, publicClients, confidentialClients } = statistics

  return (
    <div className='flex flex-col gap-6'>
      {/* Statistics Cards */}
      <div className='grid gap-4 md:grid-cols-2 lg:grid-cols-4'>
        <StatisticsCard
          title='Total Clients'
          value={totalClients}
          description='All registered clients'
          icon={Users}
          isLoading={isLoading}
        />

        <StatisticsCard
          title='Active Clients'
          value={activeClients}
          description={
            activeClients > 0 && totalClients > 0 ? (
              <span className='text-emerald-600 font-medium'>
                {((activeClients / totalClients) * 100).toFixed(0)}% enabled
              </span>
            ) : (
              'No active clients'
            )
          }
          icon={Activity}
          isLoading={isLoading}
        />

        <StatisticsCard
          title='Public Clients'
          value={publicClients}
          description='OAuth public flow'
          icon={Globe}
          isLoading={isLoading}
        />

        <StatisticsCard
          title='Confidential Clients'
          value={confidentialClients}
          description='OAuth confidential flow'
          icon={Lock}
          isLoading={isLoading}
        />
      </div>

      {/* Data Table */}
      <DataTable
        data={data}
        columns={columns}
        isLoading={isLoading}
        searchPlaceholder='Search clients by name or ID...'
        createData={{
          label: 'Create Client',
          onClick: handleCreateClient,
        }}
        searchKeys={['name', 'client_id']}
        enableSelection={true}
        enableFilters={true}
        filterFields={filterFields}
        filters={filters}
        onFiltersChange={onFiltersChange}
        onRowClick={(client) => {
          handleClickRow(client.id)
        }}
        onDeleteSelected={handleDeleteSelected}
        rowActions={[
          {
            label: 'Edit',
            icon: <Edit className='h-4 w-4' />,
            onClick: (client) => console.log('Edit', client),
          },
          {
            label: 'View',
            icon: <ExternalLink className='h-4 w-4' />,
            onClick: (client) => console.log('View', client),
          },
          {
            label: 'Delete',
            icon: <Trash2 className='h-4 w-4' />,
            variant: 'destructive',
            onClick: (client) => onRowDelete(client),
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
