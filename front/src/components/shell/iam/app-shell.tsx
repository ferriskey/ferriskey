import { useEffect, useState, type MouseEvent } from 'react'
import { Outlet, useParams } from 'react-router-dom'
import { TooltipProvider } from '@/components/ui/tooltip'
import { useGetUserRealmsQuery } from '@/api/realm.api'
import useRealmStore from '@/store/realm.store'
import { useLayoutTier } from '@/hooks/use-media-query'
import { RouterParams } from '@/routes/router'
import { TopBar } from '../top-bar'
import { REALM_URL } from '@/routes/router'
import { Sidebar } from './sidebar'
import { Crumbs } from './crumbs'
import { useCrumbs } from './use-crumbs'
import { useSidebarCollapsed } from './use-sidebar-collapsed'
import { Sheet, SheetContent } from '@/components/ui/sheet'
import { NavTree } from '../nav-tree'

export function AppShell() {
  const crumbs = useCrumbs()
  const { collapsed, toggle } = useSidebarCollapsed()
  const { realm_name } = useParams<RouterParams>()
  const { setUserRealms } = useRealmStore()
  const { data: userRealmsResponse } = useGetUserRealmsQuery({
    realm: realm_name ?? 'master',
  })
  const tier = useLayoutTier()
  const [navOpen, setNavOpen] = useState(false)
  const base = REALM_URL(realm_name ?? 'master')

  useEffect(() => {
    if (userRealmsResponse) setUserRealms(userRealmsResponse.data)
  }, [userRealmsResponse, setUserRealms])

  useEffect(() => {
    document.documentElement.dataset.style = 'ferriskey'
    return () => {
      delete document.documentElement.dataset.style
    }
  }, [])

  const closeOnNavigate = (event: MouseEvent<HTMLDivElement>) => {
    if ((event.target as Element).closest('a')) setNavOpen(false)
  }

  return (
    <TooltipProvider delayDuration={200}>
      <div className='flex h-screen flex-col bg-white dark:bg-fk-surface text-fk-ink'>
        <TopBar
          realm={realm_name ?? 'master'}
          homeHref={`${REALM_URL(realm_name ?? 'master')}/overview`}
          realmHrefFor={(name) => `${REALM_URL(name)}/overview`}
          onOpenNav={tier === 'desktop' ? undefined : () => setNavOpen(true)}
        >
          <Crumbs crumbs={crumbs} />
        </TopBar>
        <div className='flex min-h-0 flex-1'>
          {tier === 'desktop' && <Sidebar collapsed={collapsed} onToggle={toggle} />}
          <main className='min-w-0 flex-1 overflow-y-auto bg-fk-canvas'>
            <Outlet />
          </main>
        </div>
        <Sheet open={navOpen && tier !== 'desktop'} onOpenChange={setNavOpen}>
          <SheetContent label='Navigation'>
            <div onClick={closeOnNavigate}>
              <NavTree base={base} collapsed={false} />
            </div>
          </SheetContent>
        </Sheet>
      </div>
    </TooltipProvider>
  )
}
