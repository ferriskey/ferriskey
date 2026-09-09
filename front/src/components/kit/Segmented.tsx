import { cn } from '@/lib/utils'

export interface SegmentedItem {
  key: string
  label: string
}

/**
 * Bascule segmentée — un rail teinté, le segment actif en surface pleine.
 *
 * FK-17 : sert de second niveau à l'intérieur d'un onglet. Empiler deux
 * rangées d'onglets soulignés ne dit pas laquelle commande l'autre ; deux
 * formes distinctes le disent — l'onglet de page souligné, la bascule
 * interne pleine.
 */
export function Segmented({
  items,
  value,
  onChange,
  className,
}: {
  items: readonly SegmentedItem[]
  value: string
  onChange: (key: string) => void
  className?: string
}) {
  return (
    <div
      role='tablist'
      className={cn(
        'inline-flex items-center gap-1 rounded-lg bg-neutral-100 p-1',
        className
      )}
    >
      {items.map((item) => {
        const active = item.key === value
        return (
          <button
            key={item.key}
            type='button'
            role='tab'
            aria-selected={active}
            onClick={() => onChange(item.key)}
            className={cn(
              'cursor-pointer rounded-md px-3 py-1.5 text-xs font-medium transition-colors',
              'focus-visible:outline-none focus-visible:ring-3 focus-visible:ring-fk-primary/30',
              active
                ? 'bg-white text-neutral-900 shadow-sm'
                : 'text-neutral-500 hover:text-neutral-900'
            )}
          >
            {item.label}
          </button>
        )
      })}
    </div>
  )
}
