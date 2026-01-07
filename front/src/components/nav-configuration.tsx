import { Folder, ScanFace, Settings } from 'lucide-react'
import {
  SidebarGroup,
  SidebarGroupLabel,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from '@/components/ui/sidebar'
import { useNavigate, useParams } from 'react-router'
import { REALM_SETTINGS_URL, RouterParams } from '@/routes/router'
import { IDENTITY_PROVIDERS_OVERVIEW_URL } from '@/routes/sub-router/identity-providers.router'
import { USER_FEDERATION_OVERVIEW_URL } from '@/routes/sub-router/user-federation.router'

export function NavConfiguration() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()

  const handleClick = (url: string) => {
    navigate(url)
  }

  return (
    <SidebarGroup className='group-data-[collapsible=icon]:hidden'>
      <SidebarGroupLabel>Configure</SidebarGroupLabel>
      <SidebarMenu>
        <SidebarMenuItem onClick={() => handleClick(`${REALM_SETTINGS_URL(realm_name)}`)}>
          <SidebarMenuButton className='flex items-center gap-2 cursor-pointer'>
            <Settings />
            <span>Realm settings</span>
          </SidebarMenuButton>
        </SidebarMenuItem>

        <SidebarMenuItem>
          <SidebarMenuButton className='flex items-center gap-2 cursor-not-allowed text-gray-400 hover:text-gray-400'>
            <Folder />
            <span>Authentication</span>
          </SidebarMenuButton>
        </SidebarMenuItem>

        <SidebarMenuItem onClick={() => handleClick(`${IDENTITY_PROVIDERS_OVERVIEW_URL(realm_name)}`)}>
          <SidebarMenuButton className='flex items-center gap-2 cursor-pointer'>
            <ScanFace />
            <span>Identity Providers</span>
          </SidebarMenuButton>
        </SidebarMenuItem>

        <SidebarMenuItem onClick={() => handleClick(`${USER_FEDERATION_OVERVIEW_URL(realm_name)}`)}>
          <SidebarMenuButton className='flex items-center gap-2 cursor-pointer'>
            <Folder />
            <span>User Federation</span>
          </SidebarMenuButton>
        </SidebarMenuItem>
      </SidebarMenu>
    </SidebarGroup>
  )
}
