import { Role } from '@/api/core.interface'
import BadgeColor, { BadgeColorScheme } from '@/components/ui/badge-color'
import { ColumnDef } from '@/components/ui/data-table'

export const columns: ColumnDef<Role>[] = [
  {
    id: 'name',
    header: 'Name',
    cell(role) {
      return (
        <div className="flex items-center gap-2">
          <div>
            <BadgeColor color={BadgeColorScheme.PRIMARY}>{role.client?.client_id}</BadgeColor>
          </div>
          <span>{role.name}</span>
        </div>
      )
    },
  },
]
