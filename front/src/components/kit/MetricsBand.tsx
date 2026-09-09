import { ArrowUpRight } from 'lucide-react'
import { Sparkline, type ChartTone } from './charts'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'

export interface Metric {
  key: string
  label: string
  value: number | string
  hint?: string
  /** Variation sur la fenêtre observée. Omis s'il n'y a rien de mesuré. */
  delta?: number
  /** Série mesurée. Omise, la cellule bascule en variante « décomptes ». */
  series?: number[]
  tone?: ChartTone
}

/**
 * Bandeau de métriques en tête de listing — une bande unique qui garde le
 * delta et la sparkline du style data-first sans en prendre la hauteur.
 *
 * Partagé par les listings et les pages de supervision, pour qu'un chiffre se
 * lise partout de la même façon.
 */
export function MetricsBand({ metrics }: { metrics: Metric[] }) {
  if (metrics.length === 0) return null

  /* FK-12 : aucune série dans tout le bandeau — la gouttière réservée au
     graphique laisserait un vide à droite de chaque cellule. On la récupère
     pour le chiffre, qui devient le sujet de la cellule au lieu d'en partager
     la place avec un espace mort. */
  const hasSeries = metrics.some((m) => m.series)

  return (
    <div
      className={cn(
        tokens.surface.panel,
        'grid grid-cols-2 divide-fk-line overflow-hidden lg:grid-cols-4 lg:divide-x'
      )}
    >
      {metrics.map((m) =>
        hasSeries ? (
          <div key={m.key} className='flex items-center gap-3 px-3 py-2'>
            <div className='min-w-0 flex-1'>
              <p className='truncate text-[11px] text-neutral-500'>{m.label}</p>
              <div className='flex items-baseline gap-1.5'>
                <span className='tnum text-lg font-semibold leading-tight'>
                  {m.value}
                </span>
                {m.delta !== undefined && m.delta !== 0 && (
                  <span className='tnum inline-flex items-center text-[11px] text-fk-success'>
                    <ArrowUpRight className='size-3' />
                    {m.delta}
                  </span>
                )}
                {m.hint && (
                  <span className='truncate text-[11px] text-neutral-400'>
                    {m.hint}
                  </span>
                )}
              </div>
            </div>
            {m.series && (
              <div className='w-14 shrink-0'>
                <Sparkline data={m.series} tone={m.tone ?? 'info'} height={26} />
              </div>
            )}
          </div>
        ) : (
          <div key={m.key} className='px-3 py-2'>
            <p className='truncate text-[11px] text-neutral-500'>{m.label}</p>
            <div className='mt-0.5 flex items-baseline gap-1.5'>
              <span className='tnum text-xl font-semibold leading-none'>
                {m.value}
              </span>
              {m.hint && (
                <span className='truncate text-[11px] text-neutral-400'>
                  {m.hint}
                </span>
              )}
            </div>
          </div>
        )
      )}
    </div>
  )
}
