import { ChevronRight } from 'lucide-react'
import { Link } from 'react-router-dom'
import { cn } from '@/lib/utils'
import type { Crumb } from './use-crumbs'

export function Crumbs({ crumbs }: { crumbs: Crumb[] }) {
  return (
    <>
      {crumbs.map((crumb, i) => {
        const last = i === crumbs.length - 1
        return (
          <span key={`${crumb.label}-${i}`} className='flex min-w-0 items-center gap-2'>
            <ChevronRight className='size-3.5 shrink-0 text-neutral-300 dark:text-neutral-600' />
            {crumb.to && !last ? (
              <Link
                to={crumb.to}
                className='truncate rounded px-1 text-[13px] text-neutral-500 transition-colors hover:text-neutral-900 dark:text-neutral-400 dark:hover:text-neutral-100'
              >
                {crumb.label.toLowerCase()}
              </Link>
            ) : (
              <span
                className={cn(
                  'truncate px-1 text-[13px]',
                  last
                    ? 'font-medium text-neutral-900 dark:text-neutral-100'
                    : 'text-neutral-500 dark:text-neutral-400'
                )}
              >
                {crumb.label.toLowerCase()}
              </span>
            )}
          </span>
        )
      })}
    </>
  )
}
