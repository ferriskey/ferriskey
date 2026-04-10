import { ColumnDef } from '@/components/ui/data-table'
import { Schemas } from '@/api/api.client'
import User = Schemas.User

export const columns: ColumnDef<User>[] = [
  {
    id: 'username',
    header: 'Username',
    cell(user) {
      return <span className='font-medium'>{user.username}</span>
    },
  },
  {
    id: 'email',
    header: 'Email',
    cell(user) {
      return <span className='text-sm text-muted-foreground'>{user.email}</span>
    },
  },
  {
    id: 'name',
    header: 'Name',
    cell(user) {
      const name = [user.firstname, user.lastname].filter(Boolean).join(' ')
      return <span className='text-sm'>{name || '—'}</span>
    },
  },
  {
    id: 'status',
    header: 'Status',
    cell(user) {
      return (
        <span
          className={`inline-flex items-center px-2 py-0.5 rounded-md border text-xs font-mono ${
            user.enabled
              ? 'border-green-300 text-green-600 bg-green-50 dark:bg-green-500/10 dark:border-green-400/40'
              : 'border-border text-muted-foreground bg-muted/50'
          }`}
        >
          {user.enabled ? 'enabled' : 'disabled'}
        </span>
      )
    },
  },
]
