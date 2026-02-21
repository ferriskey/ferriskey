import { TriangleAlert } from 'lucide-react'
import * as React from 'react'

import { NavMain } from '@/components/nav-main'
import { NavConfiguration } from '@/components/nav-configuration'

import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarHeader,
  SidebarRail,
} from '@/components/ui/sidebar'
import { cn } from '@/lib/utils'
import { Link, useParams } from 'react-router'
import RealmSwitcher from './realm-switcher'
import { REALM_OVERVIEW_URL, REALM_URL, RouterParams } from '@/routes/router'
import { useConfig } from '@/hooks/use-config'
import BadgeColor from './ui/badge-color'
import { BadgeColorScheme } from '@/components/ui/badge-color.enum'
import { useSidebar } from './ui/sidebar-hooks'
import { NavSecurity } from './nav-security'


export function AppSidebar({ ...props }: React.ComponentProps<typeof Sidebar>) {
  const { state } = useSidebar()
  const { realm_name } = useParams<RouterParams>()
  const { config } = useConfig()

  return (
    <Sidebar variant='sidebar' collapsible='icon' {...props}>
      <SidebarHeader className='border-b border-sidebar-border px-4 py-3'>
        <Link
          className='flex items-center gap-2 cursor-pointer'
          to={`${REALM_URL(realm_name)}${REALM_OVERVIEW_URL}`}
        >
          <div className='size-8 shrink-0'>
            <img src='/logo_ferriskey.png' className='object-contain' />
          </div>
          {state === 'expanded' && (
            <>
              <span className='text-base font-bold text-sidebar-foreground tracking-tight'>FerrisKey</span>
              <ConsoleBadge />
            </>
          )}
        </Link>
      </SidebarHeader>
      <div className='border-b border-sidebar-border'>
        <RealmSwitcher />
      </div>
      <SidebarContent>
        <NavMain />
        <NavConfiguration />
        <NavSecurity />
      </SidebarContent>
      <SidebarFooter>
        {config && (
          <div className='flex flex-col gap-2 pb-1'>
            <div>
              <BadgeColor color={BadgeColorScheme.PRIMARY}>{config.app_version}</BadgeColor>
            </div>

            {config.environment === 'development' && (
              <div className='rounded-sm bg-primary/10 p-3'>
                <div className='flex'>
                  <div className='shrink-0'>
                    <TriangleAlert aria-hidden='true' className='size-4 text-primary' />
                  </div>
                  <div className='ml-2'>
                    <h3 className='text-xs font-medium text-primary'>Development mode</h3>
                  </div>
                </div>
              </div>
            )}
          </div>
        )}
      </SidebarFooter>
      <SidebarRail />
    </Sidebar>
  )
}

interface ConsoleBadgeProps {
  className?: string
}

function ConsoleBadge({ className }: ConsoleBadgeProps) {
  return (
    <div
      className={cn(
        'inline-flex items-center rounded-none bg-sidebar-foreground/10 border border-sidebar-foreground/20 px-1.5 py-0.5 text-[9px] font-bold text-sidebar-foreground/60 tracking-widest uppercase',
        className
      )}
    >
      Console
    </div>
  )
}
