import { Schemas } from '@/api/api.client'
import { DataTable, ColumnDef, RowAction } from '@/components/ui/data-table'
import { Pencil, Trash2 } from 'lucide-react'

import ProtocolMapper = Schemas.ProtocolMapper

const columns: ColumnDef<ProtocolMapper>[] = [
  {
    id: 'name',
    header: 'Name',
    accessorKey: 'name',
  },
  {
    id: 'mapper_type',
    header: 'Mapper Type',
    accessorKey: 'mapper_type',
  },
  {
    id: 'created_at',
    header: 'Created At',
    cell: (mapper) => new Date(mapper.created_at).toLocaleDateString(),
  },
]

interface PageProtocolMappersProps {
  mappers: ProtocolMapper[]
  isLoading: boolean
  onAdd: () => void
  onEdit: (mapper: ProtocolMapper) => void
  onDelete: (mapper: ProtocolMapper) => void
}

export default function PageProtocolMappers({
  mappers,
  isLoading,
  onAdd,
  onEdit,
  onDelete,
}: PageProtocolMappersProps) {
  const rowActions: RowAction<ProtocolMapper>[] = [
    {
      label: 'Edit',
      icon: <Pencil className='h-4 w-4' />,
      onClick: onEdit,
    },
    {
      label: 'Delete',
      icon: <Trash2 className='h-4 w-4' />,
      onClick: onDelete,
      variant: 'destructive',
    },
  ]

  return (
    <DataTable
      data={mappers}
      columns={columns}
      isLoading={isLoading}
      searchKeys={['name', 'mapper_type']}
      searchPlaceholder='Search protocol mappers...'
      rowActions={rowActions}
      createData={{
        label: 'Add Mapper',
        onClick: onAdd,
      }}
      emptyState={
        <span className='text-sm text-muted-foreground'>
          No protocol mappers configured. Click &ldquo;Add Mapper&rdquo; to create one.
        </span>
      }
    />
  )
}
