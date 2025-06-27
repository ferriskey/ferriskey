import { Folder, ScanFace, Settings } from 'lucide-react'
import {
  SidebarGroup,
  SidebarGroupLabel,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from '@/components/ui/sidebar'

export function NavConfiguration() {
  return (
    <SidebarGroup className="group-data-[collapsible=icon]:hidden">
      <SidebarGroupLabel>Configure</SidebarGroupLabel>
      <SidebarMenu>
        <SidebarMenuItem>
          <SidebarMenuButton className="flex items-center gap-2 cursor-not-allowed text-gray-400 hover:text-gray-400">
            <Settings />
            <span>Realm settings</span>
          </SidebarMenuButton>
        </SidebarMenuItem>

        <SidebarMenuItem>
          <SidebarMenuButton className="flex items-center gap-2 cursor-not-allowed text-gray-400 hover:text-gray-400">
            <Folder />
            <span>Authentication</span>
          </SidebarMenuButton>
        </SidebarMenuItem>

        <SidebarMenuItem>
          <SidebarMenuButton className="flex items-center gap-2 cursor-not-allowed text-gray-400 hover:text-gray-400">
            <ScanFace />
            <span>Identity Providers</span>
          </SidebarMenuButton>
        </SidebarMenuItem>
      </SidebarMenu>
    </SidebarGroup>
  )
}
