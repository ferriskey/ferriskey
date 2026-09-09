import { useEffect } from 'react'
import { Outlet, useParams } from 'react-router-dom'
import { TooltipProvider } from '@/components/ui/tooltip'
import { useGetUserRealmsQuery } from '@/api/realm.api'
import useRealmStore from '@/store/realm.store'
import { RouterParams } from '@/routes/router'
import { AccountMenu } from '../account-menu'
import { ThemeSwitcher } from '../theme-switcher'
import { NextSidebar } from './sidebar'
import { RealmBreadcrumb } from './realm-breadcrumb'
import { useCrumbs } from './use-crumbs'
import { useSidebarCollapsed } from './use-sidebar-collapsed'

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
