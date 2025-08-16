import BadgeColor from '@/components/ui/badge-color'
import { BadgeColorScheme } from '@/components/ui/badge-color.enum'
import { ColumnDef } from '@/components/ui/data-table'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
import { isServiceAccount } from '@/utils'
import { Info } from 'lucide-react'
import { Schemas } from '@/api/api.client.ts'
import User = Schemas.User

export const columns: ColumnDef<User>[] = [
  {
    id: 'name',
    header: 'User',
    cell: (user) => {
      const isSA = isServiceAccount(user)
      return (
        <div className='flex items-center gap-3'>
          <div className='h-8 w-8 rounded-full bg-primary/10 flex items-center justify-center'>
            <span className='text-xs font-medium text-primary'>
              {isSA ? 'SA' : user.firstname?.[0]?.toUpperCase() || 'U'}
            </span>
          </div>
          <div>
            {isSA ? (
              <div className='font-medium'>Service Account</div>
            ) : (
              <>
                <div className='font-medium'>
                  {user.firstname} {user.lastname}
                </div>
              </>
            )}
            <div className='text-xs text-muted-foreground'>{user.username}</div>
          </div>

          <div>
            {!user.enabled && (
              <BadgeColor color={BadgeColorScheme.RED}>
                <div className='flex items-center gap-1'>
                  <Info size={16} />
                  <span>Disabled</span>
                </div>
              </BadgeColor>
            )}
          </div>
        </div>
      )
    },
  },
  {
    id: 'email',
    header: 'Email',
    cell(user) {
      return (
        <div className='flex items-center gap-1'>
          {!user.email_verified && !user.client_id && (
            <Tooltip>
              <TooltipTrigger asChild>
                <Info size={16} className='text-red-500 cursor-pointer' />
              </TooltipTrigger>

              <TooltipContent>
                <div>Email not verified</div>
              </TooltipContent>
            </Tooltip>
          )}
          <div className='text-sm font-medium'>{user.email || '-'}</div>
        </div>
      )
    },
  },
  {
    id: 'type',
    header: 'Type',
    cell: (user) => {
      if (isServiceAccount(user)) {
        return <BadgeColor color={BadgeColorScheme.PRIMARY}>Service Account</BadgeColor>
      }
      return <BadgeColor color={BadgeColorScheme.BLUE}>User Account</BadgeColor>
    },
  },
]
