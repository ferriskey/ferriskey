import {
  SidebarGroup,
  SidebarGroupLabel,
  SidebarMenu,
  SidebarMenuBadge,
  SidebarMenuButton,
  SidebarMenuItem,
} from '@/components/ui/sidebar'
import { RouterParams } from '@/routes/router'
import { CLIENTS_URL, OVERVIEW_URL } from '@/routes/sub-router/client.router'
import { Key, LayoutGrid, ShieldCheck, Users } from 'lucide-react'
import { useLocation, useNavigate, useParams } from 'react-router'
import { ROLE_OVERVIEW_URL, ROLES_URL } from '../routes/sub-router/role.router'
import { USER_OVERVIEW_URL, USERS_URL } from '../routes/sub-router/user.router'
import { useGetClients } from '@/api/client.api'
import { useGetUsers } from '@/api/user.api'
import { useGetRoles } from '@/api/role.api'
import { cn } from '@/lib/utils'

const navItem = (active: boolean) => cn(
  'relative',
  active
    ? 'before:absolute before:left-0 before:inset-y-0 before:w-[4px] before:bg-sidebar-primary'
    : ''
)

const btnClass = (active: boolean) => cn(
  'cursor-pointer rounded-none pl-[calc(0.5rem+3px)]',
  active
    ? 'bg-sidebar-primary/15 hover:bg-sidebar-primary/20 [&>span]:text-sidebar-primary [&>svg]:text-sidebar-primary'
    : ''
)

export function NavMain() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const location = useLocation()

  const { data: clientsData } = useGetClients({ realm: realm_name ?? 'master' })
  const { data: usersData } = useGetUsers({ realm: realm_name ?? 'master' })
  const { data: rolesData } = useGetRoles({ realm: realm_name ?? 'master' })

  const handleClick = (url: string) => {
    navigate(url)
  }

  const isActive = (basePath: string) => location.pathname.startsWith(basePath)

  return (
    <SidebarGroup className='pr-2 pl-0 py-2'>
      <SidebarGroupLabel className='px-4'>Core</SidebarGroupLabel>
      <SidebarMenu>
        <SidebarMenuItem
          onClick={() => handleClick(`${CLIENTS_URL(realm_name)}${OVERVIEW_URL}`)}
          className={navItem(isActive(CLIENTS_URL(realm_name)))}
        >
          <SidebarMenuButton
            isActive={isActive(CLIENTS_URL(realm_name))}
            className={btnClass(isActive(CLIENTS_URL(realm_name)))}
          >
            <LayoutGrid />
            <span>Clients</span>
          </SidebarMenuButton>
          {clientsData?.data && (
            <SidebarMenuBadge>{clientsData.data.length}</SidebarMenuBadge>
          )}
        </SidebarMenuItem>

        <SidebarMenuItem
          onClick={() => handleClick(`${USERS_URL(realm_name)}${USER_OVERVIEW_URL}`)}
          className={navItem(isActive(USERS_URL(realm_name)))}
        >
          <SidebarMenuButton
            isActive={isActive(USERS_URL(realm_name))}
            className={btnClass(isActive(USERS_URL(realm_name)))}
          >
            <Users />
            <span>Users</span>
          </SidebarMenuButton>
          {usersData?.data && (
            <SidebarMenuBadge>{usersData.data.length}</SidebarMenuBadge>
          )}
        </SidebarMenuItem>

        <SidebarMenuItem
          onClick={() => handleClick(`${ROLES_URL(realm_name)}${ROLE_OVERVIEW_URL}`)}
          className={navItem(isActive(ROLES_URL(realm_name)))}
        >
          <SidebarMenuButton
            isActive={isActive(ROLES_URL(realm_name))}
            className={btnClass(isActive(ROLES_URL(realm_name)))}
          >
            <ShieldCheck />
            <span>Roles</span>
          </SidebarMenuButton>
          {rolesData?.data && (
            <SidebarMenuBadge>{rolesData.data.length}</SidebarMenuBadge>
          )}
        </SidebarMenuItem>

        <SidebarMenuItem>
          <SidebarMenuButton className='cursor-not-allowed rounded-none opacity-40 pl-[calc(0.5rem+2px)] hover:bg-transparent hover:text-sidebar-foreground'>
            <Key />
            <span>Client Scopes</span>
          </SidebarMenuButton>
        </SidebarMenuItem>
      </SidebarMenu>
    </SidebarGroup>
  )
}
