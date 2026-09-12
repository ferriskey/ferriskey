import { useParams } from 'react-router-dom'
import { PanelLeftClose, PanelLeftOpen } from 'lucide-react'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
import { cn } from '@/lib/utils'
import { RouterParams } from '@/routes/router'
import { REALM_URL } from '@/routes/router'
import { NavTree } from '../nav-tree'

const COLLAPSED = 'w-[3.25rem]'

export function Sidebar({
  collapsed,
  onToggle,
}: {
  collapsed: boolean
  onToggle: () => void
}) {
  const { realm_name } = useParams<RouterParams>()
  const base = REALM_URL(realm_name ?? 'master')

  return (
    <aside
      className={cn(
        'flex shrink-0 flex-col border-r border-fk-line bg-white dark:bg-fk-surface transition-[width] duration-200',
        collapsed ? COLLAPSED : 'w-44'
      )}
    >
      <NavTree base={base} collapsed={collapsed} />

      <div
        className={cn(
          'shrink-0 border-t border-fk-line',
          collapsed ? 'p-1.5' : 'p-2'
        )}
      >
        <div className={cn('flex items-center', collapsed ? 'justify-center' : 'pl-1')}>
          {!collapsed && (
            <span className='font-mono-ui text-[11px] tnum text-neutral-400 dark:text-neutral-500'>
              v{__APP_VERSION__}
            </span>
          )}
          <Tooltip>
            <TooltipTrigger asChild>
              <button
                type='button'
                onClick={onToggle}
                aria-label={collapsed ? 'Expand navigation' : 'Collapse navigation'}
                aria-expanded={!collapsed}
                className={cn(
                  'grid size-7 shrink-0 cursor-pointer place-items-center rounded-md text-neutral-400 transition-colors hover:bg-neutral-100 hover:text-neutral-900 dark:text-neutral-500 dark:hover:bg-fk-raised dark:hover:text-neutral-100',
                  !collapsed && 'ml-auto'
                )}
              >
                {collapsed ? (
                  <PanelLeftOpen className='size-4' />
                ) : (
                  <PanelLeftClose className='size-4' />
                )}
              </button>
            </TooltipTrigger>
            <TooltipContent side='right'>
              {collapsed ? 'Expand' : 'Collapse'} · ⌘B
            </TooltipContent>
          </Tooltip>
        </div>
      </div>
    </aside>
  )
}
