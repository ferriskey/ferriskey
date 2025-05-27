import {
  SidebarGroup,
  SidebarGroupLabel,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from "@/components/ui/sidebar"
import { REALM_OVERVIEW_URL, REALM_URL, RouterParams } from "@/routes/router"
import { CLIENTS_URL, OVERVIEW_URL } from "@/routes/sub-router/client.router"
import { BookOpen, Bot, SquareAsterisk } from "lucide-react"
import { useNavigate, useParams } from "react-router"
import { ROLE_OVERVIEW_URL, ROLES_URL } from '../routes/sub-router/role.router'
import { USER_OVERVIEW_URL, USER_URL } from '../routes/sub-router/user.router'

export function NavMain() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()

  const handleClick = (url: string) => {
    navigate(url)
  }

  return (
    <SidebarGroup>
      <SidebarGroupLabel onClick={() => handleClick(`${REALM_URL(realm_name)}${REALM_OVERVIEW_URL}`)}>Manage</SidebarGroupLabel>
      <SidebarMenu>
        <SidebarMenuItem onClick={() => handleClick(`${CLIENTS_URL(realm_name)}${OVERVIEW_URL}`)}>
          <SidebarMenuButton className="flex items-center gap-2 cursor-pointer">
            {/* icon */}
            <SquareAsterisk />
            <span>Clients</span>
          </SidebarMenuButton>
        </SidebarMenuItem>
        <SidebarMenuItem onClick={() => handleClick(`${USER_URL(realm_name)}${USER_OVERVIEW_URL}`)}>
          <SidebarMenuButton className="flex items-center gap-2 cursor-pointer">
            {/* icon */}
            <Bot />
            <span>Users</span>
          </SidebarMenuButton>
        </SidebarMenuItem>
        <SidebarMenuItem onClick={() => handleClick(`${ROLES_URL(realm_name)}${ROLE_OVERVIEW_URL}`)}>
          <SidebarMenuButton className="flex items-center gap-2 cursor-pointer">
            {/* icon */}
            <BookOpen />
            <span>Roles</span>
          </SidebarMenuButton>
        </SidebarMenuItem>
      </SidebarMenu>
    </SidebarGroup>
  )
}
