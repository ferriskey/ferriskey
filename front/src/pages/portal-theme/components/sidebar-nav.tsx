import { LayoutGrid, MousePointerClick, Square, TextCursorInput, Type } from 'lucide-react'
import { cn } from '@/lib/utils'
import { usePortalThemeContext, type BuilderTab } from '../context/portal-theme-context'

type Item = { id: BuilderTab; label: string; Icon: typeof Square }

const ITEMS: Item[] = [
  { id: 'buttons', label: 'Buttons', Icon: MousePointerClick },
  { id: 'inputs', label: 'Inputs', Icon: TextCursorInput },
  { id: 'widget', label: 'Widget', Icon: Square },
  { id: 'typography', label: 'Typography', Icon: Type },
  { id: 'page', label: 'Page', Icon: LayoutGrid },
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
