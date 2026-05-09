import { useGetUserRealmsQuery } from '@/api/realm.api'
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { useAuth } from '@/hooks/use-auth'
import { RouterParams } from '@/routes/router'
import useRealmStore from '@/store/realm.store'
import { deriveModeFromPath, useSwitchMode } from '@/hooks/use-switch-mode'
import { useTrackLastVisited } from '@/hooks/use-track-last-visited'
import {
  BadgeCheck,
  Check,
  ChevronDown,
  ChevronRight,
  Globe,
  Laptop,
  LayoutGrid,
  LogOut,
  Moon,
  Settings2,
  Sun,
} from 'lucide-react'
import { useEffect } from 'react'
import { Link, NavLink, Outlet, useLocation, useNavigate, useParams } from 'react-router'
import { useTheme } from '../theme-provider'
import { findActiveSection, productSections } from './product-nav-config'

function getInitials(username?: string): string {
  if (!username) return '??'
  const parts = username.trim().split(/[\s._-]+/)
  if (parts.length >= 2) return (parts[0][0] + parts[1][0]).toUpperCase()
  return username.slice(0, 2).toUpperCase()
}

export default function ProductLayout() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const { pathname } = useLocation()
  const { user, logout } = useAuth()
  const { theme, setTheme } = useTheme()
  const { setUserRealms, userRealms } = useRealmStore()
  const switchMode = useSwitchMode()
  useTrackLastVisited()
  const mode = deriveModeFromPath(pathname)

  const { data: userRealmsResponse } = useGetUserRealmsQuery({ realm: realm_name ?? 'master' })

  useEffect(() => {
    if (userRealmsResponse) setUserRealms(userRealmsResponse.data)
  }, [userRealmsResponse, setUserRealms])

  const activeRealm = realm_name ?? 'master'
  const activeSection = findActiveSection(pathname, activeRealm)

  return (
    <div className='min-h-screen bg-background flex flex-col'>
      {/* Top Bar — breadcrumb + profile */}
      <header className='sticky top-0 z-20 flex h-12 items-center gap-3 border-b border-border bg-background px-6'>
        <Link to={`/realms/${activeRealm}/overview`} className='flex items-center gap-2'>
          <img src='/logo_ferriskey.png' alt='FerrisKey' className='h-5 w-5' />
          <span className='text-sm font-semibold tracking-tight'>FerrisKey</span>
        </Link>

        <ChevronRight className='h-4 w-4 text-muted-foreground/60' />

        {/* Realm switcher */}
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <button className='inline-flex items-center gap-2 rounded-md border border-border bg-background px-2.5 py-1 text-sm hover:bg-muted transition-colors'>
              <Globe className='h-3.5 w-3.5 text-muted-foreground' />
              <span className='font-medium'>{activeRealm}</span>
              <ChevronDown className='h-3.5 w-3.5 text-muted-foreground' />
            </button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align='start' className='w-56 rounded-md'>
            <DropdownMenuLabel className='text-xs text-muted-foreground font-normal'>
              Tenants
            </DropdownMenuLabel>
            {userRealms.map((r) => (
              <DropdownMenuItem
                key={r.id}
                className='rounded-md'
                onClick={() => navigate(`/realms/${r.name}/overview`)}
              >
                <Globe className='h-3.5 w-3.5' />
                <span className='font-medium'>{r.name}</span>
                {r.name === activeRealm && <Check className='ml-auto h-4 w-4' />}
              </DropdownMenuItem>
            ))}
          </DropdownMenuContent>
        </DropdownMenu>

        {activeSection && (
          <>
            <ChevronRight className='h-4 w-4 text-muted-foreground/60' />
            <span className='text-sm text-muted-foreground'>{activeSection.label}</span>
          </>
        )}

        {/* Right: profile */}
        <div className='ml-auto flex items-center gap-2'>
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <button className='outline-none'>
                <Avatar className='size-8 rounded-md cursor-pointer'>
                  <AvatarImage src={user?.avatar} alt={user?.preferred_username} />
                  <AvatarFallback className='rounded-md bg-sidebar-primary text-sidebar-primary-foreground text-xs font-bold'>
                    {getInitials(user?.preferred_username)}
                  </AvatarFallback>
                </Avatar>
              </button>
            </DropdownMenuTrigger>
            <DropdownMenuContent className='w-56 rounded-md' align='end' sideOffset={8}>
              <DropdownMenuLabel className='p-0 font-normal'>
                <div className='flex items-center gap-2 px-2 py-2 text-left text-sm'>
                  <Avatar className='size-8 rounded-md'>
                    <AvatarImage src={user?.avatar} alt={user?.preferred_username} />
                    <AvatarFallback className='rounded-md bg-sidebar-primary text-sidebar-primary-foreground text-xs font-bold'>
                      {getInitials(user?.preferred_username)}
                    </AvatarFallback>
                  </Avatar>
                  <div className='grid flex-1 text-left text-sm leading-tight'>
                    <span className='truncate font-medium'>{user?.preferred_username}</span>
                    <span className='truncate text-xs text-muted-foreground'>{user?.email}</span>
                  </div>
                </div>
              </DropdownMenuLabel>
              <DropdownMenuSeparator />
              <DropdownMenuGroup>
                <DropdownMenuItem className='rounded-md'>
                  <BadgeCheck />
                  Account
                </DropdownMenuItem>
              </DropdownMenuGroup>
              <DropdownMenuSeparator />
              <DropdownMenuGroup>
                <DropdownMenuLabel className='font-normal text-xs text-muted-foreground px-2 py-1'>
                  Panel mode
                </DropdownMenuLabel>
                <DropdownMenuItem className='rounded-md' onClick={() => switchMode('console')}>
                  <LayoutGrid className='mr-2 h-4 w-4' />
                  Console
                  {mode === 'console' && <Check className='ml-auto h-4 w-4' />}
                </DropdownMenuItem>
                <DropdownMenuItem className='rounded-md' onClick={() => switchMode('admin')}>
                  <Settings2 className='mr-2 h-4 w-4' />
                  Admin
                  {mode === 'admin' && <Check className='ml-auto h-4 w-4' />}
                </DropdownMenuItem>
              </DropdownMenuGroup>
              <DropdownMenuSeparator />
              <DropdownMenuGroup>
                <DropdownMenuLabel className='font-normal text-xs text-muted-foreground px-2 py-1'>
                  Theme
                </DropdownMenuLabel>
                <DropdownMenuItem className='rounded-md' onClick={() => setTheme('light')}>
                  <Sun className='mr-2 h-4 w-4' />
                  Light
                  {theme === 'light' && <Check className='ml-auto h-4 w-4' />}
                </DropdownMenuItem>
                <DropdownMenuItem className='rounded-md' onClick={() => setTheme('dark')}>
                  <Moon className='mr-2 h-4 w-4' />
                  Dark
                  {theme === 'dark' && <Check className='ml-auto h-4 w-4' />}
                </DropdownMenuItem>
                <DropdownMenuItem className='rounded-md' onClick={() => setTheme('system')}>
                  <Laptop className='mr-2 h-4 w-4' />
                  System
                  {theme === 'system' && <Check className='ml-auto h-4 w-4' />}
                </DropdownMenuItem>
              </DropdownMenuGroup>
              <DropdownMenuSeparator />
              <DropdownMenuItem className='rounded-md' onClick={() => logout()}>
                <LogOut />
                Log out
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </header>

      {/* Horizontal section nav */}
      <nav className='sticky top-12 z-10 flex h-12 items-stretch gap-1 border-b border-border bg-background px-6'>
        {productSections.map((s) => {
          const isActive = activeSection?.key === s.key
          return (
            <NavLink
              key={s.key}
              to={s.to(activeRealm)}
              className={`relative inline-flex items-center gap-2 px-4 text-sm font-medium transition-colors ${
                isActive
                  ? 'text-foreground'
                  : 'text-muted-foreground hover:text-foreground'
              }`}
            >
              <s.icon className='h-4 w-4' />
              {s.label}
              {isActive && (
                <span className='absolute inset-x-3 -bottom-px h-0.5 bg-primary rounded-full' />
              )}
            </NavLink>
          )
        })}
      </nav>

      {/* Main split: vertical sub-nav + outlet */}
      <div className='flex flex-1 min-h-0'>
        {activeSection && activeSection.subItems.length > 0 && (
          <aside className='w-60 shrink-0 border-r border-border bg-muted/20 p-3'>
            <ul className='flex flex-col gap-1'>
              {activeSection.subItems.map((item) => {
                const isActive = item.match(pathname, activeRealm)
                return (
                  <li key={item.label}>
                    <NavLink
                      to={item.to(activeRealm)}
                      className={`group flex items-start gap-3 rounded-md px-3 py-2 text-sm transition-colors ${
                        isActive
                          ? 'bg-primary/10 text-primary'
                          : 'text-foreground hover:bg-muted'
                      }`}
                    >
                      <item.icon
                        className={`h-4 w-4 mt-0.5 shrink-0 ${
                          isActive ? 'text-primary' : 'text-muted-foreground group-hover:text-foreground'
                        }`}
                      />
                      <div className='flex flex-col min-w-0'>
                        <span className='font-medium truncate'>{item.label}</span>
                        {item.description && (
                          <span className='text-xs text-muted-foreground truncate'>
                            {item.description}
                          </span>
                        )}
                      </div>
                    </NavLink>
                  </li>
                )
              })}
            </ul>
          </aside>
        )}

        <main className='flex-1 min-w-0 overflow-x-hidden'>
          <Outlet />
        </main>
      </div>
    </div>
  )
}
