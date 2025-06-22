import { Role } from '@/api/api.interface'
import { ColumnDef } from '@/components/ui/data-table'

export const columns: ColumnDef<Role>[] = [
  {
    id: 'name',
    header: 'Name',
    cell(role) {
      return (
        <div>
          <span>{role.name}</span>
        </div>
      )
    },
  },
]
