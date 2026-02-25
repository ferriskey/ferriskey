import { Eye } from 'lucide-react'
import { SidebarGroup, SidebarGroupLabel, SidebarMenu, SidebarMenuButton, SidebarMenuItem } from './ui/sidebar'
import { useLocation, useNavigate, useParams } from 'react-router-dom'
import { REALM_URL, RouterParams } from '@/routes/router'
import { SEAWATCH_OVERVIEW_URL } from '@/routes/sub-router/seawatch.router'
import { cn } from '@/lib/utils'

const navItem = (active: boolean) => cn(
  'relative',
  active ? 'before:absolute before:left-0 before:inset-y-0 before:w-[3px] before:bg-sidebar-primary' : ''
)

const btnClass = (active: boolean) => cn(
  'cursor-pointer rounded-none pl-[calc(0.5rem+3px)]',
  active ? 'bg-sidebar-primary/15 hover:bg-sidebar-primary/20 [&>span]:text-sidebar-primary [&>svg]:text-sidebar-primary' : ''
)

export function NavSecurity() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const location = useLocation()

  const seawatchBasePath = `${REALM_URL(realm_name)}/seawatch`
  const active = location.pathname.startsWith(seawatchBasePath)

  return (
    <SidebarGroup className='pr-2 pl-0 py-2'>
      <SidebarGroupLabel className='px-4'>Security</SidebarGroupLabel>

      <SidebarMenu>
        <SidebarMenuItem onClick={() => navigate(SEAWATCH_OVERVIEW_URL(realm_name))} className={navItem(active)}>
          <SidebarMenuButton isActive={active} className={btnClass(active)}>
            <Eye />
            <span>Sea Watch</span>
          </SidebarMenuButton>
        </SidebarMenuItem>
      </SidebarMenu>
    </SidebarGroup>
  )
}
