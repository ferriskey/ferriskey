import { Brush, Palette, Type } from 'lucide-react'
import { cn } from '@/lib/utils'
import { usePortalThemeContext, type BuilderTab } from '../context/portal-theme-context'

type Item = { id: BuilderTab; label: string; Icon: typeof Palette }

const ITEMS: Item[] = [
  { id: 'colors', label: 'Colors', Icon: Palette },
  { id: 'fonts', label: 'Fonts', Icon: Type },
  { id: 'borders', label: 'Borders & widget', Icon: Brush },
]

export function SidebarNav() {
  const { activeTab, setActiveTab } = usePortalThemeContext()

  return (
    <nav className='flex flex-col gap-1 p-2'>
      {ITEMS.map(({ id, label, Icon }) => {
        const active = activeTab === id
        return (
          <button
            key={id}
            type='button'
            onClick={() => setActiveTab(id)}
            className={cn(
              'flex items-center gap-2 rounded-md px-3 py-2 text-sm transition-colors',
              active ? 'bg-sidebar-primary/15 text-sidebar-primary' : 'hover:bg-muted',
            )}
          >
            <Icon className='h-4 w-4' />
            <span>{label}</span>
          </button>
        )
      })}
    </nav>
  )
}
