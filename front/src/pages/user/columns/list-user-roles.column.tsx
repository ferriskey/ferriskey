import BadgeColor from '@/components/ui/badge-color'
import { BadgeColorScheme } from '@/components/ui/badge-color.enum'
import { ColumnDef } from '@/components/ui/data-table'
import { Schemas } from '@/api/api.client'
import Role = Schemas.Role

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
