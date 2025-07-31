import { DataTable } from '@/components/ui/data-table'
import { Edit, ExternalLink, Trash2 } from 'lucide-react'
import { columns } from '../columns/list-client.column'
import { Client } from '@/api/core.interface'

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
  return (
    <div>
      <DataTable
        data={data}
        columns={columns}
        isLoading={isLoading}
        searchPlaceholder="Rechercher un client..."
        createData={{
          label: 'Create Client',
          onClick: handleCreateClient,
        }}
        searchKeys={['name', 'client_id']}
        enableSelection={true}
        onRowClick={(client) => {
          handleClickRow(client.id)
        }}
        onDeleteSelected={handleDeleteSelected}
        rowActions={[
          {
            label: 'Edit',
            icon: <Edit className="h-4 w-4" />,
            onClick: (client) => console.log('Edit', client),
          },
          {
            label: 'View',
            icon: <ExternalLink className="h-4 w-4" />,
            onClick: (client) => console.log('View', client),
          },
          {
            label: 'Delete',
            icon: <Trash2 className="h-4 w-4" />,
            variant: 'destructive',
            onClick: (client) => handleDeleteClient(client.id),
          },
        ]}
      />
    </div>
  )
}
