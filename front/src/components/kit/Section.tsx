import type { ReactNode } from 'react'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'

interface SectionProps {
  title: string
  description?: string
  action?: ReactNode
  children: ReactNode
  contained?: boolean
  /** Rule between children. Off when they already read as separate blocks. */
  divided?: boolean
  className?: string
}

export function Section({
  title,
  description,
  action,
  children,
  contained = true,
  divided = true,
  className,
}: SectionProps) {
  return (
    <section className={className}>
      <div className='flex items-end justify-between gap-4 pb-2'>
        <div className='min-w-0'>
          <h2 className='text-sm font-semibold text-neutral-900 dark:text-neutral-100'>{title}</h2>
          {description && (
            <p className='mt-0.5 text-xs text-neutral-500 dark:text-neutral-400'>{description}</p>
          )}
        </div>
        {action && <div className='shrink-0'>{action}</div>}
      </div>

      {contained ? (
        <div className={cn(tokens.surface.panel, 'px-4', divided && tokens.surface.divider)}>
          {children}
        </div>
      ) : (
        children
      )}
    </section>
  )
}
