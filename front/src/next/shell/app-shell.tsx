import { useEffect } from 'react'
import { Outlet, useLocation, useNavigate, useParams } from 'react-router-dom'
import {
  BadgeCheck,
  Check,
  LayoutGrid,
  LogOut,
  Settings2,
  Sparkles,
} from 'lucide-react'
import { TooltipProvider } from '@/components/ui/tooltip'
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
import { deriveModeFromPath, useSwitchMode } from '@/hooks/use-switch-mode'
import { useGetUserRealmsQuery } from '@/api/realm.api'
import useRealmStore from '@/store/realm.store'
import { RouterParams } from '@/routes/router'
import { REALM_URL } from '@/routes/router'
import { NEXT_ACCOUNT_URL } from '../routes'
import { NextSidebar } from './sidebar'
import { ThemeSwitcher } from './theme-switcher'
import { RealmBreadcrumb } from './realm-breadcrumb'
import { useCrumbs } from './use-crumbs'
import { useSidebarCollapsed } from './use-sidebar-collapsed'

function getInitials(username?: string): string {
  if (!username) return '??'
  const parts = username.trim().split(/[\s._-]+/)
  if (parts.length >= 2) return (parts[0][0] + parts[1][0]).toUpperCase()
  return username.slice(0, 2).toUpperCase()
}

function AccountMenu() {
  const { user, logout } = useAuth()
  const navigate = useNavigate()
  const { pathname } = useLocation()
  const { realm_name = 'master' } = useParams<RouterParams>()
  const switchMode = useSwitchMode()
  const mode = deriveModeFromPath(pathname)

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
          onClick={() => navigate(NEXT_ACCOUNT_URL(realm_name))}
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
            onClick={() => switchMode('console')}
          >
            <LayoutGrid className='size-4' />
            Console
            {mode === 'console' && <Check className='ml-auto size-4' />}
          </DropdownMenuItem>
          <DropdownMenuItem
            className='gap-2 px-1.5 py-1 text-[13px]'
            onClick={() => navigate(REALM_URL(realm_name))}
          >
            <Settings2 className='size-4' />
            Admin
          </DropdownMenuItem>
          <DropdownMenuItem className='gap-2 px-1.5 py-1 text-[13px]' disabled>
            <Sparkles className='size-4' />
            Admin · next
            <Check className='ml-auto size-4' />
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

export function NextAppShell() {
  const crumbs = useCrumbs()
  const { collapsed, toggle } = useSidebarCollapsed()
  const { realm_name } = useParams<RouterParams>()
  const { setUserRealms } = useRealmStore()
  const { data: userRealmsResponse } = useGetUserRealmsQuery({
    realm: realm_name ?? 'master',
  })

  useEffect(() => {
    if (userRealmsResponse) setUserRealms(userRealmsResponse.data)
  }, [userRealmsResponse, setUserRealms])

  useEffect(() => {
    document.documentElement.dataset.style = 'ferriskey'
    return () => {
      delete document.documentElement.dataset.style
    }
  }, [])

  return (
    <TooltipProvider delayDuration={200}>
      <div className='flex h-screen flex-col bg-white dark:bg-fk-surface text-fk-ink'>
        <RealmBreadcrumb
          crumbs={crumbs}
          actions={
            <>
              <ThemeSwitcher />
              <AccountMenu />
            </>
          }
        />
        <div className='flex min-h-0 flex-1'>
          <NextSidebar collapsed={collapsed} onToggle={toggle} />
          <main className='min-w-0 flex-1 overflow-y-auto bg-neutral-50/40 dark:bg-fk-surface/40'>
            <Outlet />
          </main>
        </div>
      </div>
    </TooltipProvider>
  )
}
