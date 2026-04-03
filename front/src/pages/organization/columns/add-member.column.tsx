import { ColumnDef } from '@/components/ui/data-table'
import { Schemas } from '@/api/api.client'
import User = Schemas.User

export const columns: ColumnDef<User>[] = [
  {
    id: 'username',
    header: 'Username',
    cell(user) {
      return (
        <div className='flex flex-col'>
          <span className='font-medium'>{user.username}</span>
          <span className='text-xs text-muted-foreground'>{user.email}</span>
        </div>
      )
    },
  },
  {
    id: 'name',
    header: 'Name',
    cell(user) {
      const name = [user.firstname, user.lastname].filter(Boolean).join(' ')
      return <span className='text-sm text-muted-foreground'>{name || '—'}</span>
    },
  },
]
