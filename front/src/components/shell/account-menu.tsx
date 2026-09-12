import { useNavigate, useParams } from 'react-router-dom'
import {
  BadgeCheck,
  Check,
  LayoutGrid,
  LogOut,
  Settings2,
} from 'lucide-react'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { useAuth } from '@/hooks/use-auth'
import { RouterParams } from '@/routes/router'
import { ACCOUNT_URL } from '@/routes/router'
import { usePanel, useSwitchPanel } from './ciam/use-console-switch'
import { getInitials } from './initials'

export function AccountMenu() {
  const { user, logout } = useAuth()
  const navigate = useNavigate()
  const { realm_name = 'master' } = useParams<RouterParams>()
  const panel = usePanel()
  const switchPanel = useSwitchPanel()

  if (!user) return null

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <button
          type='button'
          aria-label='Account'
          className='grid size-8 cursor-pointer place-items-center rounded-md bg-fk-brand text-xs font-semibold text-white'
        >
          {getInitials(user.preferred_username)}
        </button>
      </DropdownMenuTrigger>

      <DropdownMenuContent align='end' sideOffset={6} className='w-56 p-1'>
        <DropdownMenuLabel className='flex items-center gap-2 px-1.5 py-1.5 font-normal'>
          <span className='grid size-8 shrink-0 place-items-center rounded-md bg-fk-brand text-xs font-semibold text-white'>
            {getInitials(user.preferred_username)}
          </span>
          <span className='grid min-w-0 flex-1'>
            <span className='truncate text-[13px] font-medium text-neutral-900 dark:text-neutral-100'>
              {user.preferred_username}
            </span>
            <span className='truncate text-[11px] text-neutral-500 dark:text-neutral-400'>{user.email}</span>
          </span>
        </DropdownMenuLabel>

        <DropdownMenuSeparator />

        <DropdownMenuItem
          className='gap-2 px-1.5 py-1 text-[13px]'
          onClick={() => navigate(ACCOUNT_URL(realm_name))}
        >
          <BadgeCheck className='size-4' />
          Account
        </DropdownMenuItem>

        <DropdownMenuSeparator />

        <DropdownMenuGroup>
          <DropdownMenuLabel className='px-1.5 py-1 text-[11px] font-medium text-neutral-400 dark:text-neutral-500'>
            Panel mode
          </DropdownMenuLabel>
          <DropdownMenuItem
            className='gap-2 px-1.5 py-1 text-[13px]'
            onClick={() => switchPanel('admin')}
          >
            <Settings2 className='size-4' />
            Admin
            {panel === 'admin' && <Check className='ml-auto size-4' />}
          </DropdownMenuItem>
          <DropdownMenuItem
            className='gap-2 px-1.5 py-1 text-[13px]'
            onClick={() => switchPanel('console')}
          >
            <LayoutGrid className='size-4' />
            Console
            {panel === 'console' && <Check className='ml-auto size-4' />}
          </DropdownMenuItem>
        </DropdownMenuGroup>

        <DropdownMenuSeparator />

        <DropdownMenuItem className='gap-2 px-1.5 py-1 text-[13px]' onClick={logout}>
          <LogOut className='size-4' />
          Log out
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  )
}
