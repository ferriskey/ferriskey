import { useMemo } from 'react'
import { useLocation } from 'react-router-dom'
import type { TabItem } from './Tabs'

/**
 * Ancre l'onglet actif dans l'URL (FK-16).
 *
 * Le segment qui suit `basePath` désigne l'onglet : /users/u-2/credentials.
 * Chaque onglet devient donc une adresse à part entière — partageable,
 * ajoutable aux favoris, ouvrable dans un nouvel onglet du navigateur, et
 * restaurée telle quelle au rechargement.
 *
 * Un segment inconnu ou absent retombe sur le premier onglet, ce qui rend
 * /users/u-2 et /users/u-2/overview équivalents : une ancienne URL ne casse
 * jamais.
 */
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
