import { ColumnDef } from '@/components/ui/data-table'
import { Schemas } from '@/api/api.client'
import Organization = Schemas.Organization

export const columns: ColumnDef<Organization>[] = [
  {
    id: 'name',
    header: 'Name',
    cell(org) {
      return (
        <div className='flex flex-col'>
          <span>{org.name}</span>
          <span className='font-mono text-xs text-muted-foreground'>{org.alias}</span>
        </div>
      )
    },
  },
  {
    id: 'domain',
    header: 'Domain',
    cell(org) {
      return <span className='text-sm text-muted-foreground'>{org.domain ?? '—'}</span>
    },
  },
]
