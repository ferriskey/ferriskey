import { ColumnDef } from '@/components/ui/data-table'
import { Schemas } from '@/api/api.client'
import Organization = Schemas.Organization

export const columns: ColumnDef<Organization>[] = [
  {
    id: 'name',
    header: 'Name',
    cell(org) {
      return <span>{org.name}</span>
    },
  },
  {
    id: 'alias',
    header: 'Alias',
    cell(org) {
      return <span className='font-mono text-sm text-muted-foreground'>{org.alias}</span>
    },
  },
  {
    id: 'domain',
    header: 'Domain',
    cell(org) {
      return <span className='text-sm text-muted-foreground'>{org.domain ?? '—'}</span>
    },
  },
  {
    id: 'status',
    header: 'Status',
    cell(org) {
      return (
        <span
          className={`inline-flex items-center px-2 py-0.5 rounded-md border text-xs font-mono ${
            org.enabled
              ? 'border-green-300 text-green-600 bg-green-50 dark:bg-green-500/10 dark:border-green-400/40'
              : 'border-border text-muted-foreground bg-muted/50'
          }`}
        >
          {org.enabled ? 'enabled' : 'disabled'}
        </span>
      )
    },
  },
]
