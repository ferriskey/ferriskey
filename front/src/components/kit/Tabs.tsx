import type { ReactNode } from 'react'
import { Link } from 'react-router-dom'
import { cn } from '@/lib/utils'

export interface TabItem {
  key: string
  label: string
  /**
   * Rend l'onglet comme un lien plutôt qu'un bouton. Fourni par
   * `useRouteTabs` : l'onglet devient alors partageable, ajoutable aux
   * favoris et ouvrable dans un nouvel onglet du navigateur (FK-16).
   */
  href?: string
  count?: number
  /** Signale une anomalie sur la section (pastille ambre). */
  warn?: boolean
}

interface TabsProps {
  tabs: readonly TabItem[]
  value: string
  /** Inutile lorsque les onglets sont des liens. */
  onChange?: (key: string) => void
  /** Omis, le composant ne rend que la barre. */
  children?: ReactNode
  className?: string
}

/**
 * Onglets de ressource, partagés par toutes les pages de détail.
 *
 * Souligné : trait sous le libellé actif, filet de base sur toute la largeur.
 * Ce composant existe pour que ce motif ne soit plus recopié à la main dans
 * chaque page.
 *
 * Nommé `PageTabs` à l'export : `components/ui/tabs.tsx` (shadcn) reste en
 * place pour les bascules internes, et deux `Tabs` importables se confondent.
 */
export function PageTabs({
  tabs,
  value,
  onChange,
  children,
  className,
}: TabsProps) {
  return (
    <div className={className}>
      <div role='tablist' className='flex gap-6 border-b border-fk-line'>
        {tabs.map((tab) => {
          const active = tab.key === value
          const inner = (
            <>
              {tab.label}
              {tab.warn && (
                <span className='ml-1.5 size-1.5 rounded-full bg-fk-amber' />
              )}
              {tab.count !== undefined && (
                <span
                  className={cn(
                    'tnum ml-1.5 text-[11px]',
                    active ? 'text-fk-primary-text' : 'text-neutral-400'
                  )}
                >
                  {tab.count}
                </span>
              )}
            </>
          )

          const shape = cn(
            '-mb-px flex cursor-pointer items-center border-b-2 pb-2 text-[13px] transition-colors',
            active
              ? 'border-fk-brand font-medium text-neutral-900'
              : 'border-transparent text-neutral-500 hover:text-neutral-900'
          )

          return tab.href ? (
            <Link
              key={tab.key}
              to={tab.href}
              role='tab'
              aria-selected={active}
              className={shape}
            >
              {inner}
            </Link>
          ) : (
            <button
              key={tab.key}
              type='button'
              role='tab'
              aria-selected={active}
              onClick={() => onChange?.(tab.key)}
              className={shape}
            >
              {inner}
            </button>
          )
        })}
      </div>
      {children !== undefined && <div className='mt-3'>{children}</div>}
    </div>
  )
}
