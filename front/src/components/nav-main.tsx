import { BookOpen, Bot, SquareAsterisk } from "lucide-react"
import {
  SidebarGroup,
  SidebarGroupLabel,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from "@/components/ui/sidebar"
import { useNavigate, useParams } from "react-router"
import { CLIENT_OVERVIEW_URL, CLIENT_URL } from "@/routes/sub-router/client.router"
import { REALM_OVERVIEW_URL, REALM_URL, RouterParams } from "@/routes/router"

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
        <SidebarMenuItem onClick={() => handleClick(`${CLIENT_URL(realm_name)}${CLIENT_OVERVIEW_URL}`)}>
          <SidebarMenuButton>
            {/* icon */}
            <SquareAsterisk />
            <span>Clients</span>
          </SidebarMenuButton>
        </SidebarMenuItem>
        <SidebarMenuItem onClick={() => handleClick(`${CLIENT_URL(realm_name)}${CLIENT_OVERVIEW_URL}`)}>
          <SidebarMenuButton>
            {/* icon */}
            <Bot />
            <span>Users</span>
          </SidebarMenuButton>
        </SidebarMenuItem>
        <SidebarMenuItem onClick={() => handleClick(`${CLIENT_URL(realm_name)}${CLIENT_OVERVIEW_URL}`)}>
          <SidebarMenuButton>
            {/* icon */}
            <BookOpen />
            <span>Roles</span>
          </SidebarMenuButton>
        </SidebarMenuItem>
        {/* {items.map((item) => (
          <Collapsible
            key={item.title}
            asChild
            defaultOpen={item.isActive}
            className="group/collapsible"
          >
            <SidebarMenuItem>
              <CollapsibleTrigger 
                asChild 
                onClick={() => {
                  if (!item.items || item.items.length === 0) {
                    navigate(item.url)
                  }
                }}
              >
                <SidebarMenuButton tooltip={item.title}>
                  {item.icon && <item.icon />}
                  <span>{item.title}</span>
                  {item.items && item.items.length > 0 && (
                    <ChevronRight className="ml-auto transition-transform duration-200 group-data-[state=open]/collapsible:rotate-90" />
                  )}
                  
                </SidebarMenuButton>
              </CollapsibleTrigger>
              {item.items && item.items.length > 0 && (
                <CollapsibleContent>
                <SidebarMenuSub>
                  {item.items?.map((subItem) => (
                    <SidebarMenuSubItem key={subItem.title}>
                      <SidebarMenuSubButton asChild>
                        <a href={subItem.url}>
                          <span>{subItem.title}</span>
                        </a>
                      </SidebarMenuSubButton>
                    </SidebarMenuSubItem>
                  ))}
                </SidebarMenuSub>
              </CollapsibleContent>
              )}
              
            </SidebarMenuItem>
          </Collapsible>
        ))} */}
      </SidebarMenu>
    </SidebarGroup>
  )
}
