import { useTranslation } from 'react-i18next'
import { NavLink, useLocation } from 'react-router-dom'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
import { cn } from '@/lib/utils'
import { navSections, navSectionKey, navSegmentKey, type NavItem } from './iam/nav'

function ItemBody({
  item,
  label,
  active,
  collapsed,
}: {
  item: NavItem
  label: string
  active: boolean
  collapsed: boolean
}) {
  return (
    <>
      {active && !collapsed && (
        <span className='absolute -left-2 top-1 h-[calc(100%-0.5rem)] w-[2.5px] rounded-full bg-fk-brand' />
      )}
      <item.icon className='size-3.5 shrink-0' strokeWidth={1.75} />
      {!collapsed && <span className='min-w-0 flex-1 truncate'>{label}</span>}
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
  const { t } = useTranslation()
  const { pathname } = useLocation()
  const to = `${base}/${item.to}`
  const active = pathname.startsWith(to)
  const label = t(navSegmentKey(item.to))

  const shape = cn(
    'group relative flex items-center gap-2 rounded-md text-[13px] transition-colors',
    collapsed ? 'justify-center px-0 py-1.5' : 'px-2 py-1'
  )

  if (item.disabled) {
    return (
      <Tooltip>
        <TooltipTrigger asChild>
          <span className={cn(shape, 'cursor-not-allowed text-neutral-300 dark:text-neutral-600')} aria-disabled>
            <ItemBody item={item} label={label} active={false} collapsed={collapsed} />
          </span>
        </TooltipTrigger>
        <TooltipContent side='right'>{t('nav.not_implemented', { label })}</TooltipContent>
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
      <ItemBody item={item} label={label} active={active} collapsed={collapsed} />
    </NavLink>
  )

  if (!collapsed) return link

  return (
    <Tooltip>
      <TooltipTrigger asChild>{link}</TooltipTrigger>
      <TooltipContent side='right'>{label}</TooltipContent>
    </Tooltip>
  )
}

export function NavTree({ base, collapsed }: { base: string; collapsed: boolean }) {
  const { t } = useTranslation()

  return (
    <nav
      className={cn(
        'flex-1 overflow-y-auto overflow-x-hidden py-2',
        collapsed ? 'px-1.5' : 'px-2',
      )}
    >
      {navSections.map((section, i) => (
        <div key={section.key} className={cn('space-y-0.5', i > 0 && 'mt-5')}>
          {!collapsed && (
            <p className='px-2 pb-1 text-[11px] font-medium text-neutral-400 dark:text-neutral-500'>
              {t(navSectionKey(section.key))}
            </p>
          )}
          {section.items.map((item) => (
            <SidebarLink key={item.to} item={item} base={base} collapsed={collapsed} />
          ))}
        </div>
      ))}
    </nav>
  )
}
