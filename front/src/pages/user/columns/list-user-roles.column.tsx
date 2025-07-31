import { Role } from '@/api/core.interface'
import BadgeColor, { BadgeColorScheme } from '@/components/ui/badge-color'
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
  {
    id: 'client',
    header: 'Client',
    cell(item) {
      return (
        <div>
          <BadgeColor color={BadgeColorScheme.PRIMARY}>{item.client?.client_id}</BadgeColor>
        </div>
      )
    },
  },
]
