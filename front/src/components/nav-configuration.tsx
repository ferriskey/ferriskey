import { Database, Link2, Settings, Shield } from 'lucide-react'
import {
  SidebarGroup,
  SidebarGroupLabel,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from '@/components/ui/sidebar'
import { useLocation, useNavigate, useParams } from 'react-router-dom'
import { REALM_SETTINGS_URL, REALM_URL, RouterParams } from '@/routes/router'
import { IDENTITY_PROVIDERS_OVERVIEW_URL } from '@/routes/sub-router/identity-providers.router'
import { USER_FEDERATION_URL } from '@/routes/sub-router/user-federation.router'
import { cn } from '@/lib/utils'

const navItem = (active: boolean) => cn(
  'relative',
  active ? 'before:absolute before:left-0 before:inset-y-0 before:w-[3px] before:bg-sidebar-primary' : ''
)

const btnClass = (active: boolean) => cn(
  'cursor-pointer rounded-none pl-[calc(0.5rem+3px)]',
  active ? 'bg-sidebar-primary/15 hover:bg-sidebar-primary/20 [&>span]:text-sidebar-primary [&>svg]:text-sidebar-primary' : ''
)

export function NavConfiguration() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const location = useLocation()

  const handleClick = (url: string) => {
    navigate(url)
  }

  const isActive = (basePath: string) => location.pathname.startsWith(basePath)

  return (
    <SidebarGroup className='group-data-[collapsible=icon]:hidden pr-2 pl-0 py-2'>
      <SidebarGroupLabel className='px-4'>Configuration</SidebarGroupLabel>
      <SidebarMenu>
        <SidebarMenuItem onClick={() => handleClick(REALM_SETTINGS_URL(realm_name))} className={navItem(isActive(REALM_SETTINGS_URL(realm_name)))}>
          <SidebarMenuButton isActive={isActive(REALM_SETTINGS_URL(realm_name))} className={btnClass(isActive(REALM_SETTINGS_URL(realm_name)))}>
            <Settings />
            <span>Realm Settings</span>
          </SidebarMenuButton>
        </SidebarMenuItem>

        <SidebarMenuItem>
          <SidebarMenuButton className='cursor-not-allowed rounded-none opacity-40 pl-[calc(0.5rem+2px)] hover:bg-transparent hover:text-sidebar-foreground'>
            <Shield />
            <span>Authentication</span>
          </SidebarMenuButton>
        </SidebarMenuItem>

        <SidebarMenuItem onClick={() => handleClick(IDENTITY_PROVIDERS_OVERVIEW_URL(realm_name))} className={navItem(isActive(`${REALM_URL(realm_name)}/identity-providers`))}>
          <SidebarMenuButton isActive={isActive(`${REALM_URL(realm_name)}/identity-providers`)} className={btnClass(isActive(`${REALM_URL(realm_name)}/identity-providers`))}>
            <Link2 />
            <span>Identity Providers</span>
          </SidebarMenuButton>
        </SidebarMenuItem>

        <SidebarMenuItem onClick={() => handleClick(`${USER_FEDERATION_URL(realm_name)}/overview`)} className={navItem(isActive(USER_FEDERATION_URL(realm_name)))}>
          <SidebarMenuButton isActive={isActive(USER_FEDERATION_URL(realm_name))} className={btnClass(isActive(USER_FEDERATION_URL(realm_name)))}>
            <Database />
            <span>User Federation</span>
          </SidebarMenuButton>
        </SidebarMenuItem>
      </SidebarMenu>
    </SidebarGroup>
  )
}
