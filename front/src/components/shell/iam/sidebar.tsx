import { NavLink, useLocation, useParams } from 'react-router-dom'
import { PanelLeftClose, PanelLeftOpen } from 'lucide-react'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
import { cn } from '@/lib/utils'
import { RouterParams } from '@/routes/router'
import { REALM_URL } from '@/routes/router'
import { navSections, type NavItem } from './nav'

const COLLAPSED = 'w-[3.25rem]'

function ItemBody({
  item,
  active,
  collapsed,
}: {
  item: NavItem
  active: boolean
  collapsed: boolean
}) {
  return (
    <>
      {active && !collapsed && (
        <span className='absolute -left-2 top-1 h-[calc(100%-0.5rem)] w-[2.5px] rounded-full bg-fk-brand' />
      )}
      <item.icon className='size-3.5 shrink-0' strokeWidth={1.75} />
      {!collapsed && <span className='min-w-0 flex-1 truncate'>{item.label}</span>}
    </>
  )
}

function SidebarLink({
  item,
  base,
  collapsed,
}: {
  item: NavItem
  base: string
  collapsed: boolean
}) {
  const { pathname } = useLocation()
  const to = `${base}/${item.to}`
  const active = pathname.startsWith(to)

  const shape = cn(
    'group relative flex items-center gap-2 rounded-md text-[13px] transition-colors',
    collapsed ? 'justify-center px-0 py-1.5' : 'px-2 py-1'
  )

  if (item.disabled) {
    return (
      <Tooltip>
        <TooltipTrigger asChild>
          <span className={cn(shape, 'cursor-not-allowed text-neutral-300 dark:text-neutral-600')} aria-disabled>
            <ItemBody item={item} active={false} collapsed={collapsed} />
          </span>
        </TooltipTrigger>
        <TooltipContent side='right'>{item.label} — not implemented yet</TooltipContent>
      </Tooltip>
    )
  }

  const link = (
    <NavLink
      to={to}
      className={cn(
        shape,
        active
          ? 'bg-fk-primary-soft font-medium text-fk-primary-text'
          : 'text-neutral-700 hover:bg-neutral-100 dark:text-neutral-300 dark:hover:bg-fk-raised'
      )}
    >
      <ItemBody item={item} active={active} collapsed={collapsed} />
    </NavLink>
  )

  if (!collapsed) return link

  return (
    <Tooltip>
      <TooltipTrigger asChild>{link}</TooltipTrigger>
      <TooltipContent side='right'>{item.label}</TooltipContent>
    </Tooltip>
  )
}

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
      <nav
        className={cn(
          'flex-1 overflow-y-auto overflow-x-hidden py-2',
          collapsed ? 'px-1.5' : 'px-2'
        )}
      >
        {navSections.map((section, i) => (
          <div key={section.title} className={cn('space-y-0.5', i > 0 && 'mt-5')}>
            {!collapsed && (
              <p className='px-2 pb-1 text-[11px] font-medium text-neutral-400 dark:text-neutral-500'>
                {section.title}
              </p>
            )}
            {section.items.map((item) => (
              <SidebarLink
                key={item.to}
                item={item}
                base={base}
                collapsed={collapsed}
              />
            ))}
          </div>
        ))}
      </nav>

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
