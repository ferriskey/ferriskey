import { DataTable } from '@/components/ui/data-table'
import { Edit, ExternalLink, Trash2, Users, Globe, Lock, Activity } from 'lucide-react'
import { columns } from '../columns/list-client.column'
import { Schemas } from '@/api/api.client.ts'
import { useConfirmDeleteAlert } from '@/hooks/use-confirm-delete-alert.ts'
import { ConfirmDeleteAlert } from '@/components/confirm-delete-alert'
import { Filter, FilterFieldsConfig } from '@/components/ui/filters'
import { useState, useMemo } from 'react'
import { Card, CardContent } from '@/components/ui/card'
import { Button } from '@/components/ui/button'

import Client = Schemas.Client

export interface PageClientsOverviewProps {
  isLoading?: boolean
  data: Client[]
  realmName: string
  handleDeleteSelected: (items: Client[]) => void
  handleClickRow: (clientId: string) => void
  handleDeleteClient: (clientId: string) => void
  handleCreateClient: () => void
}

export default function PageClientsOverview({
  data,
  isLoading,
  handleDeleteSelected,
  handleClickRow,
  handleDeleteClient,
  handleCreateClient,
}: PageClientsOverviewProps) {

  const { confirm, ask, close } = useConfirmDeleteAlert()
  const [filters, setFilters] = useState<Filter[]>([])

  // Configuration des champs de filtrage
  const filterFields: FilterFieldsConfig = [
    {
      key: 'name',
      label: 'Client Name',
      type: 'text',
    },
    {
      key: 'client_id',
      label: 'Client ID',
      type: 'text',
    },
    {
      key: 'public_client',
      label: 'Type',
      type: 'boolean',
    },
    {
      key: 'enabled',
      label: 'Status',
      type: 'boolean',
    },
  ]

  // Appliquer les filtres sur les données
  const filteredData = useMemo(() => {
    if (filters.length === 0) return data

    return data.filter((client) => {
      return filters.every((filter) => {
        const fieldValue = client[filter.field as keyof Client]
        const filterValues = filter.values

        switch (filter.operator) {
          case 'is':
            return fieldValue === filterValues[0]
          case 'isNot':
            return fieldValue !== filterValues[0]
          case 'contains':
            return String(fieldValue).toLowerCase().includes(String(filterValues[0]).toLowerCase())
          case 'notContains':
            return !String(fieldValue).toLowerCase().includes(String(filterValues[0]).toLowerCase())
          case 'startsWith':
            return String(fieldValue).toLowerCase().startsWith(String(filterValues[0]).toLowerCase())
          case 'endsWith':
            return String(fieldValue).toLowerCase().endsWith(String(filterValues[0]).toLowerCase())
          case 'empty':
            return !fieldValue || fieldValue === ''
          case 'notEmpty':
            return fieldValue && fieldValue !== ''
          default:
            return true
        }
      })
    })
  }, [data, filters])

  const onRowDelete = (client: Client) => {
    ask({
      title: 'Delete client?',
      description: `Are you sure you want to delete "${client.name}"?`,
      onConfirm: () => {
        handleDeleteClient(client.id)
        close()
      },
    })
  }

  // Calcul des statistiques
  const totalClients = data.length
  const activeClients = data.filter((client) => client.enabled).length
  const publicClients = data.filter((client) => client.public_client).length
  const confidentialClients = data.filter((client) => !client.public_client).length

  return (
    <div className='flex flex-col gap-6'>
      {/* Statistics Cards */}
      <div className='grid gap-4 md:grid-cols-2 lg:grid-cols-4'>
        <Card className='hover:shadow-md transition-shadow'>
          <CardContent>
            <div className='flex items-center justify-between'>
              <div className='text-sm font-medium text-muted-foreground'>Total Clients</div>
              <div className='rounded-lg bg-muted p-2'>
                <Users className='h-5 w-5 text-muted-foreground' />
              </div>
            </div>
            <div className='mt-4'>
              <div className='text-4xl font-bold'>{totalClients}</div>
              <p className='text-sm text-muted-foreground mt-2'>
                All registered clients
              </p>
            </div>
          </CardContent>
        </Card>

        <Card className='hover:shadow-md transition-shadow'>
          <CardContent>
            <div className='flex items-center justify-between'>
              <div className='text-sm font-medium text-muted-foreground'>Active Clients</div>
              <div className='rounded-lg bg-muted p-2'>
                <Activity className='h-5 w-5 text-muted-foreground' />
              </div>
            </div>
            <div className='mt-4'>
              <div className='text-4xl font-bold'>{activeClients}</div>
              <p className='text-sm text-muted-foreground mt-2'>
                {activeClients > 0 && totalClients > 0 && (
                  <span className='text-emerald-600 font-medium'>
                    {((activeClients / totalClients) * 100).toFixed(0)}% enabled
                  </span>
                )}
                {activeClients === 0 && 'No active clients'}
              </p>
            </div>
          </CardContent>
        </Card>

        <Card className='hover:shadow-md transition-shadow'>
          <CardContent>
            <div className='flex items-center justify-between'>
              <div className='text-sm font-medium text-muted-foreground'>Public Clients</div>
              <div className='rounded-lg bg-muted p-2'>
                <Globe className='h-5 w-5 text-muted-foreground' />
              </div>
            </div>
            <div className='mt-4'>
              <div className='text-4xl font-bold'>{publicClients}</div>
              <p className='text-sm text-muted-foreground mt-2'>
                OAuth public flow
              </p>
            </div>
          </CardContent>
        </Card>

        <Card className='hover:shadow-md transition-shadow'>
          <CardContent>
            <div className='flex items-center justify-between'>
              <div className='text-sm font-medium text-muted-foreground'>Confidential Clients</div>
              <div className='rounded-lg bg-muted p-2'>
                <Lock className='h-5 w-5 text-muted-foreground' />
              </div>
            </div>
            <div className='mt-4'>
              <div className='text-4xl font-bold'>{confidentialClients}</div>
              <p className='text-sm text-muted-foreground mt-2'>
                OAuth confidential flow
              </p>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Data Table */}
      <DataTable
        data={filteredData}
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
        onFiltersChange={setFilters}
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
        onCancel={close}
      />
    </div>
  )
}
