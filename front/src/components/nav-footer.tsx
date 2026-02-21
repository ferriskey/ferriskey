import { Activity, Bell } from 'lucide-react'
import {
  SidebarMenu,
  SidebarMenuBadge,
  SidebarMenuButton,
  SidebarMenuItem,
} from '@/components/ui/sidebar'

export function NavFooter() {
  return (
    <SidebarMenu className='mb-1'>
      <SidebarMenuItem>
        <SidebarMenuButton className='cursor-pointer'>
          <Activity />
          <span>Server Health</span>
        </SidebarMenuButton>
      </SidebarMenuItem>

      <SidebarMenuItem>
        <SidebarMenuButton className='cursor-pointer'>
          <Bell />
          <span>Alerts</span>
        </SidebarMenuButton>
        <SidebarMenuBadge className='bg-primary text-primary-foreground rounded-none px-1.5'>
          7
        </SidebarMenuBadge>
      </SidebarMenuItem>
    </SidebarMenu>
  )
}
