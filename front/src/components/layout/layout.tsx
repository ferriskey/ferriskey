import { Outlet } from 'react-router'
import { AppSidebar } from '../app-sidebar'
import { SidebarInset, SidebarProvider } from '../ui/sidebar'

export default function Layout() {
  return (
    <SidebarProvider>
      <AppSidebar />
      <SidebarInset>
        <Outlet />
      </SidebarInset>
    </SidebarProvider>
  )
}
