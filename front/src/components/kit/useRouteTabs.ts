import { useMemo } from 'react'
import { useLocation } from 'react-router-dom'
import type { TabItem } from './Tabs'

export function useRouteTabs<T extends readonly TabItem[]>(
  basePath: string,
  tabs: T
): { value: string; tabs: TabItem[] } {
  const { pathname } = useLocation()

  return useMemo(() => {
    const rest = pathname.startsWith(basePath)
      ? pathname.slice(basePath.length).replace(/^\/+/, '')
      : ''
    const segment = rest.split('/')[0]
    const match = tabs.find((t) => t.key === segment)

    return {
      value: match?.key ?? tabs[0].key,
      tabs: tabs.map((t) => ({ ...t, href: `${basePath}/${t.key}` })),
    }
  }, [pathname, basePath, tabs])
}
