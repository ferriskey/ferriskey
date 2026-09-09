import type { ComponentType, ReactNode } from 'react'
import { Inbox } from 'lucide-react'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'

export interface EmptyStateProps {
  icon?: ComponentType<{ className?: string; strokeWidth?: number }>
  label: string
  hint?: string
  action?: ReactNode
  /**
   * `amber` when the emptiness has a consequence the reader should weigh,
   * `neutral` when it is merely a state.
   */
  tone?: 'neutral' | 'amber'
  /** Half the vertical room, for an empty section inside a populated page. */
  compact?: boolean
  className?: string
}

const tones = {
  neutral: {
    surface: tokens.surface.panel,
    icon: 'text-neutral-300',
    label: 'text-neutral-700',
    hint: 'text-neutral-500',
  },
  amber: {
    surface: 'rounded-sm border border-fk-amber-border bg-fk-amber-soft/40',
    icon: 'text-fk-amber',
    label: 'text-neutral-900',
    hint: 'text-neutral-600',
  },
} as const

export function EmptyState({
  icon: Icon = Inbox,
  label,
  hint,
  action,
  tone = 'neutral',
  compact = false,
  className,
}: EmptyStateProps) {
  const t = tones[tone]

  return (
    <div
      className={cn(
        t.surface,
        'grid place-items-center px-6',
        compact ? 'py-8' : 'py-16',
        className
      )}
    >
      <Icon className={cn('size-6', t.icon)} strokeWidth={1.5} />
      <p className={cn('mt-3 text-sm font-medium', t.label)}>{label}</p>
      {hint && (
        <p className={cn('mt-1 max-w-sm text-center text-sm', t.hint)}>{hint}</p>
      )}
      {action && <div className='mt-4'>{action}</div>}
    </div>
  )
}
