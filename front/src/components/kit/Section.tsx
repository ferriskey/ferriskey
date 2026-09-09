import type { ReactNode } from 'react'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'

interface SectionProps {
  title: string
  description?: string
  /** Bouton ou lien aligné à droite du titre. */
  action?: ReactNode
  children: ReactNode
  /**
   * `false` sort le contenu du panneau — réservé aux blocs qui portent déjà
   * leur propre surface (zone de danger, encart d'alerte). FK-07 : un seul
   * niveau d'encart, un panneau dans un panneau se lit comme un défaut de
   * rendu.
   */
  contained?: boolean
  className?: string
}

/**
 * Bloc de contenu d'une page de détail : titre, action facultative, et un
 * panneau qui contient le corps.
 *
 * FK-10 : le titre vit **au-dessus** du panneau. Une barre de titre à
 * l'intérieur ajoute un filet horizontal par section ; au-dessus, le titre
 * sépare sans tracer et les panneaux de la page s'alignent visuellement.
 */
export function Section({
  title,
  description,
  action,
  children,
  contained = true,
  className,
}: SectionProps) {
  return (
    <section className={className}>
      <div className='flex items-end justify-between gap-4 pb-2'>
        <div className='min-w-0'>
          <h2 className='text-sm font-semibold text-neutral-900'>{title}</h2>
          {description && (
            <p className='mt-0.5 text-xs text-neutral-500'>{description}</p>
          )}
        </div>
        {action && <div className='shrink-0'>{action}</div>}
      </div>

      {contained ? (
        <div className={cn(tokens.surface.panel, 'px-4', tokens.surface.divider)}>
          {children}
        </div>
      ) : (
        children
      )}
    </section>
  )
}
