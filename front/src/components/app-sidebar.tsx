import {
  AudioWaveform,
  BookOpen,
  Bot,
  Command,
  Frame,
  GalleryVerticalEnd,
  Map,
  PieChart,
  SquareTerminal,
} from 'lucide-react'
import * as React from 'react'

import { NavMain } from '@/components/nav-main'
import { NavProjects } from '@/components/nav-projects'
import { NavUser } from '@/components/nav-user'

import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarHeader,
  SidebarRail,
  useSidebar,
} from '@/components/ui/sidebar'
import { cn } from '@/lib/utils'
import { Link, useParams } from 'react-router'
import RealmSwitcher from './realm-switcher'
import { REALM_OVERVIEW_URL, REALM_URL, RouterParams } from '@/routes/router'
import { CLIENT_OVERVIEW_URL, CLIENT_URL } from '@/routes/sub-router/client.router'
import { USER_OVERVIEW_URL, USER_URL } from '@/routes/sub-router/user.router'

// This is sample data.
const data = {
  user: {
    name: 'shadcn',
    email: 'm@example.com',
    avatar: '/avatars/shadcn.jpg',
  },
  teams: [
    {
      name: 'Acme Inc',
      logo: GalleryVerticalEnd,
      plan: 'Enterprise',
    },
    {
      name: 'Acme Corp.',
      logo: AudioWaveform,
      plan: 'Startup',
    },
    {
      name: 'Evil Corp.',
      logo: Command,
      plan: 'Free',
    },
  ],
  navMain: [
    {
      title: 'Clients',
      url: `${CLIENT_URL('master')}${CLIENT_OVERVIEW_URL}`,
      icon: SquareTerminal,
      isActive: true,
    },
    {
      title: 'Users',
      url: `${USER_URL('master')}${USER_OVERVIEW_URL}`,
      icon: Bot,
    },
    {
      title: 'Roles',
      url: '#',
      icon: BookOpen,
    },
  ],
  projects: [
    {
      name: 'Realm Settings',
      url: '#',
      icon: Frame,
    },
    {
      name: 'Authentication',
      url: '#',
      icon: PieChart,
    },
    {
      name: 'Identity Providers',
      url: '#',
      icon: Map,
    },
  ],
}

export function AppSidebar({ ...props }: React.ComponentProps<typeof Sidebar>) {
  const { state } = useSidebar()
  const { realm_name } = useParams<RouterParams>()

  return (
    <Sidebar variant="inset" collapsible="icon" {...props}>
      <SidebarHeader>
      <Link
          className={cn('flex items-center gap-3 cursor-pointer', state === 'expanded' && 'hover:bg-gray-100 rounded-md')}
          to={`${REALM_URL(realm_name)}${REALM_OVERVIEW_URL}`}
        >
          <div className='flex items-center gap-2'>
            <div className="size-12">
              <img src="/logo_ferriskey.png" />
            </div>
            <div className={cn(state === 'collapsed' ? 'hidden' : 'flex')} >
              <span className='text-lg font-medium text-gray-600'>FerrisKey</span>
            </div>
          </div>
          <ConsoleBadge className={cn(state === 'collapsed' ? 'hidden' : 'flex')} />
        </Link>
        <RealmSwitcher />
      </SidebarHeader>
      <SidebarContent>
        <NavMain />
        <NavProjects projects={data.projects} />
      </SidebarContent>
      <SidebarFooter>
        <NavUser user={data.user} />
      </SidebarFooter>
      <SidebarRail />
    </Sidebar>
  )
}

interface ConsoleBadgeProps {
  className?: string
}

function ConsoleBadge({ className }: ConsoleBadgeProps) {
  return (
    <div
      className={cn(
        "inline-flex items-center rounded-[2px] bg-zinc-900 px-2 py-0.5 text-xs font-medium text-white",
        className,
      )}
    >
      Console
    </div>
  )
}
