import { ScanEye } from "lucide-react";
import { SidebarGroup, SidebarGroupLabel, SidebarMenu, SidebarMenuButton, SidebarMenuItem } from "./ui/sidebar";
import { useNavigate, useParams } from "react-router";
import { RouterParams } from "@/routes/router";
import { SEAWATCH_OVERVIEW_URL } from "@/routes/sub-router/seawatch.router";

export function NavSecurity() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()

  const handleClick = (url: string) => {
    navigate(url)
  }

  return (
    <SidebarGroup>
      <SidebarGroupLabel>Security</SidebarGroupLabel>

      <SidebarMenu>
        <SidebarMenuItem onClick={() => handleClick(`${SEAWATCH_OVERVIEW_URL(realm_name)}`)}>
          <SidebarMenuButton>
            <ScanEye />
            <span>Sea Watch</span>
          </SidebarMenuButton>
        </SidebarMenuItem>
      </SidebarMenu>
    </SidebarGroup >
  )
}
