import { useEffect } from 'react'
import { Outlet, useNavigate, useParams } from 'react-router-dom'
import { LogOut } from 'lucide-react'
import { TooltipProvider } from '@/components/ui/tooltip'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { useAuth } from '@/hooks/use-auth'
import { useGetUserRealmsQuery } from '@/api/realm.api'
import useRealmStore from '@/store/realm.store'
import { RouterParams } from '@/routes/router'
import { REALM_URL } from '@/routes/router'
import { NEXT_ACCOUNT_URL } from '../routes'
import { NextSidebar } from './sidebar'
import { RealmBreadcrumb } from './realm-breadcrumb'
import { useCrumbs } from './use-crumbs'
import { useSidebarCollapsed } from './use-sidebar-collapsed'

function AccountMenu() {
  const { user, logout } = useAuth()
  const navigate = useNavigate()
  const { realm_name = 'master' } = useParams<RouterParams>()

  if (!user) return null

  const initials = (user.preferred_username ?? '?').slice(0, 2).toUpperCase()

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <button
          type='button'
          aria-label='Account'
          className='grid size-8 cursor-pointer place-items-center rounded-md bg-fk-brand text-xs font-semibold text-white'
        >
          {initials}
        </button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align='end' className='w-56'>
        <DropdownMenuLabel className='font-normal'>
          <span className='block truncate text-sm font-medium'>
            {user.preferred_username}
          </span>
          <span className='block truncate text-xs text-neutral-500'>{user.email}</span>
        </DropdownMenuLabel>
        <DropdownMenuSeparator />
        <DropdownMenuItem onClick={() => navigate(NEXT_ACCOUNT_URL(realm_name))}>
          My account
        </DropdownMenuItem>
        <DropdownMenuItem onClick={() => navigate(REALM_URL(realm_name))}>
          Back to the current console
        </DropdownMenuItem>
        <DropdownMenuSeparator />
        <DropdownMenuItem onClick={logout}>
          <LogOut /> Log out
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
      <div className='flex h-screen flex-col bg-white text-fk-ink'>
        <RealmBreadcrumb crumbs={crumbs} actions={<AccountMenu />} />
        <div className='flex min-h-0 flex-1'>
          <NextSidebar collapsed={collapsed} onToggle={toggle} />
          <main className='min-w-0 flex-1 overflow-y-auto bg-neutral-50/40'>
            <Outlet />
          </main>
        </div>
      </div>
    </TooltipProvider>
  )
}
