import { useEffect } from 'react'
import { ChevronRight } from 'lucide-react'
import { NavLink, Outlet, useLocation, useParams } from 'react-router-dom'
import { TooltipProvider } from '@/components/ui/tooltip'
import { useGetUserRealmsQuery } from '@/api/realm.api'
import useRealmStore from '@/store/realm.store'
import { RouterParams } from '@/routes/router'
import { cn } from '@/lib/utils'
import { TopBar } from '../top-bar'
import {
  consoleSections,
  findActiveSection,
  isSubItemActive,
  sectionUrl,
  subItemUrl,
  NEXT_CONSOLE_URL,
} from './nav-config'

export function ConsoleShell() {
  const { realm_name = 'master' } = useParams<RouterParams>()
  const { pathname } = useLocation()
  const { setUserRealms } = useRealmStore()
  const { data: userRealmsResponse } = useGetUserRealmsQuery({ realm: realm_name })

  useEffect(() => {
    if (userRealmsResponse) setUserRealms(userRealmsResponse.data)
  }, [userRealmsResponse, setUserRealms])

  useEffect(() => {
    document.documentElement.dataset.style = 'ferriskey'
    return () => {
      delete document.documentElement.dataset.style
    }
  }, [])

  const section = findActiveSection(pathname, realm_name)

  return (
    <TooltipProvider delayDuration={200}>
      <div className='flex h-screen flex-col overflow-hidden bg-white text-fk-ink dark:bg-fk-surface'>
        <TopBar
          realm={realm_name}
          homeHref={`${NEXT_CONSOLE_URL(realm_name)}/activity/live`}
          realmHrefFor={(name) => `${NEXT_CONSOLE_URL(name)}/activity/live`}
        >
          {section && (
            <>
              <ChevronRight className='hidden size-3.5 shrink-0 text-neutral-300 md:block dark:text-neutral-600' />
              <span className='hidden truncate text-[13px] text-neutral-500 md:inline dark:text-neutral-400'>
                {section.label.toLowerCase()}
              </span>
            </>
          )}
        </TopBar>

        <nav className='flex h-9 shrink-0 items-stretch gap-1 border-b border-fk-line px-3'>
          {consoleSections.map((s) => {
            const active = section?.key === s.key
            return (
              <NavLink
                key={s.key}
                to={sectionUrl(realm_name, s)}
                className={cn(
                  'relative inline-flex shrink-0 items-center gap-1.5 px-2.5 text-[13px] transition-colors',
                  active
                    ? 'font-medium text-neutral-900 dark:text-neutral-100'
                    : 'text-neutral-500 hover:text-neutral-900 dark:text-neutral-400 dark:hover:text-neutral-100'
                )}
              >
                <s.icon className='size-3.5' strokeWidth={1.75} />
                {s.label}
                {active && (
                  <span className='absolute inset-x-2.5 -bottom-px h-0.5 rounded-full bg-fk-brand' />
                )}
              </NavLink>
            )
          })}
        </nav>

        <div className='flex min-h-0 flex-1 items-stretch'>
          {section && section.subItems.length > 1 && (
            <aside className='hidden w-52 shrink-0 overflow-y-auto border-r border-fk-line p-2 md:block'>
              <ul className='flex flex-col gap-0.5'>
                {section.subItems.map((item) => {
                  const active = isSubItemActive(pathname, realm_name, item)
                  return (
                    <li key={item.key}>
                      <NavLink
                        to={subItemUrl(realm_name, item)}
                        className={cn(
                          'group flex items-start gap-2 rounded-md px-2 py-1.5 transition-colors',
                          active
                            ? 'bg-fk-primary-soft text-fk-primary-text'
                            : 'text-neutral-700 hover:bg-neutral-100 dark:text-neutral-300 dark:hover:bg-fk-raised'
                        )}
                      >
                        <item.icon className='mt-0.5 size-3.5 shrink-0' strokeWidth={1.75} />
                        <span className='min-w-0'>
                          <span className='block truncate text-[13px] font-medium'>
                            {item.label}
                          </span>
                          <span
                            className={cn(
                              'block truncate text-[11px]',
                              active
                                ? 'text-fk-primary-text/80'
                                : 'text-neutral-500 dark:text-neutral-400'
                            )}
                          >
                            {item.description}
                          </span>
                        </span>
                      </NavLink>
                    </li>
                  )
                })}
              </ul>
            </aside>
          )}

          <main className='min-w-0 flex-1 overflow-y-auto bg-neutral-50/40 dark:bg-neutral-900/40'>
            <Outlet />
          </main>
        </div>
      </div>
    </TooltipProvider>
  )
}
