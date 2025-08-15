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
        <div className='flex items-center gap-2'>
          <div>
            <BadgeColor color={BadgeColorScheme.PRIMARY}>{role.client?.client_id}</BadgeColor>
          </div>
          <span>{role.name}</span>
        </div>
      )
    },
  },
]
