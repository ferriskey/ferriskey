import type { ComponentType, ReactNode } from 'react'
import { Inbox } from 'lucide-react'
import { cn } from '@/lib/utils'

export interface EmptyStateProps {
  icon?: ComponentType<{ className?: string; strokeWidth?: number }>
  label: string
  hint?: string
  action?: ReactNode
  /** Half the vertical room, for an empty section inside a populated page. */
  compact?: boolean
  className?: string
}

export function EmptyState({
  icon: Icon = Inbox,
  label,
  hint,
  action,
  compact = false,
  className,
}: EmptyStateProps) {
  return (
    <div
      className={cn(
        'grid place-items-center rounded-sm border border-dashed border-fk-line bg-white dark:bg-neutral-900 px-6',
        compact ? 'py-8' : 'py-16',
        className
      )}
    >
      <Icon className='size-6 text-neutral-300 dark:text-neutral-600' strokeWidth={1.5} />
      <p className='mt-3 text-sm font-medium text-neutral-700 dark:text-neutral-300'>{label}</p>
      {hint && (
        <p className='mt-1 max-w-sm text-center text-sm text-neutral-500 dark:text-neutral-400'>{hint}</p>
      )}
      {action && <div className='mt-4'>{action}</div>}
    </div>
  )
}
