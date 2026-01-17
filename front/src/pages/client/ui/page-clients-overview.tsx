import { DataTable } from '@/components/ui/data-table'
import { Edit, ExternalLink, Trash2 } from 'lucide-react'
import { columns } from '../columns/list-client.column'
import { Schemas } from '@/api/api.client.ts'
import { ConfirmDeleteAlert } from '@/components/confirm-delete-alert'
import { Filter, FilterFieldsConfig } from '@/components/ui/filters'
import StatisticsCard from '../components/statistics-card'
import ClientTrendChart from '../components/client-trend-chart'

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
          isLoading={isLoading}
          chart={
            !isLoading && <ClientTrendChart clients={data} days={7} color='hsl(200 76% 36%)' />
          }
        />

        <StatisticsCard
          title='Active Clients'
          value={activeClients}
          isLoading={isLoading}
          trend={{
            value:
              activeClients > 0 && totalClients > 0
                ? Math.round((activeClients / totalClients) * 100)
                : 0,
            direction: 'up',
          }}
          chart={
            !isLoading && (
              <ClientTrendChart
                clients={data.filter((c) => c.enabled)}
                days={7}
                color='hsl(142 76% 36%)'
              />
            )
          }
        />

        <StatisticsCard
          title='Public Clients'
          value={publicClients}
          isLoading={isLoading}
          chart={
            !isLoading && (
              <ClientTrendChart
                clients={data.filter((c) => c.public_client)}
                days={7}
                color='hsl(221 83% 53%)'
              />
            )
          }
        />

        <StatisticsCard
          title='Confidential Clients'
          value={confidentialClients}
          isLoading={isLoading}
          chart={
            !isLoading && (
              <ClientTrendChart
                clients={data.filter((c) => !c.public_client)}
                days={7}
                color='hsl(262 83% 58%)'
              />
            )
          }
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
            onClick: (client) => handleClickRow(client.id),
          },
          {
            label: 'View',
            icon: <ExternalLink className='h-4 w-4' />,
            onClick: (client) => handleClickRow(client.id),
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
