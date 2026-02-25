import { Bell, Search } from 'lucide-react'
import { useParams } from 'react-router-dom'
import { useAuth } from '@/hooks/use-auth'
import { RouterParams } from '@/routes/router'
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
import { BadgeCheck, LogOut, Sun, Moon, Laptop, Check } from 'lucide-react'
import { useTheme } from './theme-provider'

function getInitials(username?: string): string {
  if (!username) return '??'
  const parts = username.trim().split(/[\s._-]+/)
  if (parts.length >= 2) {
    return (parts[0][0] + parts[1][0]).toUpperCase()
  }
  return username.slice(0, 2).toUpperCase()
}

export function TopBar() {
  const { realm_name } = useParams<RouterParams>()
  const { user, logout } = useAuth()
  const { theme, setTheme } = useTheme()

  return (
    <header className='sticky top-0 z-10 flex h-14 items-center gap-4 border-b border-border bg-background px-6'>
      {/* Search */}
      <div className='flex flex-1 items-center gap-2 rounded-none border border-input bg-muted/40 px-3 max-w-sm focus-within:border-sidebar-primary transition-colors'>
        <Search className='size-3.5 shrink-0 text-muted-foreground' />
        <input
          type='text'
          placeholder='Search realm...'
          className='flex-1 bg-transparent py-1.5 text-sm text-foreground placeholder:text-muted-foreground outline-none'
        />
        <kbd className='hidden sm:inline-flex h-5 items-center gap-1 rounded-none border border-border bg-background px-1.5 font-mono text-[10px] font-medium text-muted-foreground'>
          ⌘K
        </kbd>
      </div>

      <div className='ml-auto flex items-center gap-4'>
        {/* Realm indicator */}
        {realm_name && (
          <div className='hidden sm:flex items-center gap-1.5 rounded-none bg-muted px-2.5 py-1 text-xs font-mono text-muted-foreground border border-border'>
            <span>realm:</span>
            <span className='text-foreground font-medium'>{realm_name}</span>
          </div>
        )}

        {/* Bell */}
        <button className='relative flex size-8 items-center justify-center rounded-none text-muted-foreground hover:text-foreground transition-colors'>
          <Bell className='size-4' />
        </button>

        {/* Avatar dropdown */}
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <button className='outline-none'>
              <Avatar className='size-8 rounded-none cursor-pointer'>
                <AvatarImage src={user?.avatar} alt={user?.preferred_username} />
                <AvatarFallback className='rounded-none bg-sidebar-primary text-sidebar-primary-foreground text-xs font-bold'>
                  {getInitials(user?.preferred_username)}
                </AvatarFallback>
              </Avatar>
            </button>
          </DropdownMenuTrigger>
          <DropdownMenuContent className='w-56 rounded-none' align='end' sideOffset={8}>
            <DropdownMenuLabel className='p-0 font-normal'>
              <div className='flex items-center gap-2 px-2 py-2 text-left text-sm'>
                <Avatar className='size-8 rounded-none'>
                  <AvatarImage src={user?.avatar} alt={user?.preferred_username} />
                  <AvatarFallback className='rounded-none bg-sidebar-primary text-sidebar-primary-foreground text-xs font-bold'>
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
              <DropdownMenuItem className='rounded-none'>
                <BadgeCheck />
                Account
              </DropdownMenuItem>
            </DropdownMenuGroup>
            <DropdownMenuSeparator />
            <DropdownMenuGroup>
              <DropdownMenuLabel className='font-normal text-xs text-muted-foreground px-2 py-1'>Theme</DropdownMenuLabel>
              <DropdownMenuItem className='rounded-none' onClick={() => setTheme('light')}>
                <Sun className='mr-2 h-4 w-4' />
                Light
                {theme === 'light' && <Check className='ml-auto h-4 w-4' />}
              </DropdownMenuItem>
              <DropdownMenuItem className='rounded-none' onClick={() => setTheme('dark')}>
                <Moon className='mr-2 h-4 w-4' />
                Dark
                {theme === 'dark' && <Check className='ml-auto h-4 w-4' />}
              </DropdownMenuItem>
              <DropdownMenuItem className='rounded-none' onClick={() => setTheme('system')}>
                <Laptop className='mr-2 h-4 w-4' />
                System
                {theme === 'system' && <Check className='ml-auto h-4 w-4' />}
              </DropdownMenuItem>
            </DropdownMenuGroup>
            <DropdownMenuSeparator />
            <DropdownMenuItem className='rounded-none' onClick={() => logout()}>
              <LogOut />
              Log out
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>
    </header>
  )
}
