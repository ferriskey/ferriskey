import { TriangleAlert } from 'lucide-react'
import * as React from 'react'

import { NavMain } from '@/components/nav-main'
import { NavConfiguration } from '@/components/nav-configuration'
import { NavUser } from '@/components/nav-user'

import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarHeader,
  SidebarRail,
  useSidebar,
} from '@/components/ui/sidebar'
import { cn } from '@/lib/utils'
import { Link, useParams } from 'react-router'
import RealmSwitcher from './realm-switcher'
import { REALM_OVERVIEW_URL, REALM_URL, RouterParams } from '@/routes/router'
import { useConfig } from '@/hooks/use-config'
import BadgeColor, { BadgeColorScheme } from './ui/badge-color'

export function AppSidebar({ ...props }: React.ComponentProps<typeof Sidebar>) {
  const { state } = useSidebar()
  const { realm_name } = useParams<RouterParams>()
  const { config } = useConfig()

  return (
    <Sidebar variant="inset" collapsible="icon" {...props}>
      <SidebarHeader>
        <Link
          className={cn(
            'flex items-center gap-3 cursor-pointer',
            state === 'expanded' && 'hover:bg-gray-100 rounded-md'
          )}
          to={`${REALM_URL(realm_name)}${REALM_OVERVIEW_URL}`}
        >
          <div className="flex items-center gap-2">
            <div className="size-12">
              <img src="/logo_ferriskey.png" />
            </div>
            <div className={cn(state === 'collapsed' ? 'hidden' : 'flex')}>
              <span className="text-lg font-medium text-gray-600">FerrisKey</span>
            </div>
          </div>
          <ConsoleBadge className={cn(state === 'collapsed' ? 'hidden' : 'flex')} />
        </Link>
        <RealmSwitcher />
      </SidebarHeader>
      <SidebarContent>
        <NavMain />
        <NavConfiguration />
      </SidebarContent>
      <SidebarFooter>
        {config && (
          <div className="flex flex-col gap-2">
            <div>
              <BadgeColor color={BadgeColorScheme.PRIMARY}>{config.app_version}</BadgeColor>
            </div>

            {config.environment === 'development' && (
              <div className="rounded-md bg-primary/10 p-4">
                <div className="flex">
                  <div className="shrink-0">
                    <TriangleAlert aria-hidden="true" className="size-5 text-primary" />
                  </div>
                  <div className="ml-3">
                    <h3 className="text-sm font-medium text-primary">Development mode</h3>
                    <div className="mt-2 text-sm text-primary/75">
                      <p>You are currently in development mode.</p>
                    </div>
                  </div>
                </div>
              </div>
            )}
          </div>
        )}
        <NavUser />
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
        'inline-flex items-center rounded-[2px] bg-zinc-900 px-2 py-0.5 text-xs font-medium text-white',
        className
      )}
    >
      Console
    </div>
  )
}
