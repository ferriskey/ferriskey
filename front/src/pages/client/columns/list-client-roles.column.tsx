import { Schemas } from '@/api/api.client'
import { ColumnDef } from '@/components/ui/data-table'
import Role = Schemas.Role

export const columns: ColumnDef<Role>[] = [
  {
    id: 'Name',
    header: 'Nom du rôle',
    cell: (role) => <div className='font-medium'>{role.name}</div>,
  },
  {
    id: 'Description',
    header: 'Description',
    cell: (role) => <div className='text-muted-foreground'>{role.description || '-'}</div>,
  },
  {
    id: 'Created At',
    header: 'Créé le',
    cell: (role) => (
      <div className='text-muted-foreground'>{new Date(role.created_at).toLocaleDateString()}</div>
    ),
  },
]
