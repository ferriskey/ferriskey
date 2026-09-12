import type { ReactNode } from 'react'
import { cn } from '@/lib/utils'

export type PillTone =
  | 'neutral'
  | 'success'
  | 'info'
  | 'violet'
  | 'amber'
  | 'danger'
  | 'primary'

const pillTones: Record<PillTone, string> = {
  neutral: 'border-fk-line bg-neutral-50 text-neutral-600 dark:bg-fk-surface dark:text-neutral-400',
  success: 'border-fk-success-border bg-fk-success-soft text-fk-success',
  info: 'border-fk-info-border bg-fk-info-soft text-fk-info',
  violet: 'border-fk-violet-border bg-fk-violet-soft text-fk-violet',
  amber: 'border-fk-amber-border bg-fk-amber-soft text-fk-amber',
  danger: 'border-fk-danger-border bg-fk-danger-soft text-fk-danger',
  primary: 'border-fk-primary-border bg-fk-primary-soft text-fk-primary-text',
}

export function Pill({
  tone = 'neutral',
  mono,
  className,
  children,
}: {
  tone?: PillTone
  mono?: boolean
  className?: string
  children: ReactNode
}) {
  return (
    <span
      className={cn(
        'inline-flex items-center gap-1 rounded border px-1.5 py-0.5 text-xs leading-5',
        mono && 'font-mono-ui',
        pillTones[tone],
        className
      )}
    >
      {children}
    </span>
  )
}

export function StatusDot({ on }: { on: boolean }) {
  return (
    <span
      className={cn(
        'inline-block size-1.5 rounded-full',
        on ? 'bg-fk-success' : 'bg-neutral-300'
      )}
    />
  )
}

const squircleTones = [
  'bg-fk-brand',
  'bg-fk-primary',
  'bg-fk-info',
  'bg-fk-success',
  'bg-fk-violet',
  'bg-fk-amber',
]

export function Squircle({
  name,
  size = 'md',
  className,
}: {
  name: string
  size?: 'sm' | 'md' | 'lg' | 'xl'
  className?: string
}) {
  const tone =
    squircleTones[
      name.split('').reduce((a, c) => a + c.charCodeAt(0), 0) %
        squircleTones.length
    ]

  return (
    <span
      className={cn(
        'grid shrink-0 place-items-center rounded-md font-semibold uppercase text-white',
        size === 'sm' && 'size-6 text-[11px]',
        size === 'md' && 'size-9 text-sm',
        size === 'lg' && 'size-11 text-base',
        size === 'xl' && 'size-15 text-2xl',
        tone,
        className
      )}
    >
      {name.charAt(0)}
    </span>
  )
}

export function Eyebrow({ children }: { children: ReactNode }) {
  return (
    <p className='text-[11px] font-semibold uppercase tracking-[0.14em] text-neutral-400 dark:text-neutral-500'>
      {children}
    </p>
  )
}

const iconTileTones = {
  info: 'bg-fk-info-soft text-fk-info',
  success: 'bg-fk-success-soft text-fk-success',
  violet: 'bg-fk-violet-soft text-fk-violet',
  amber: 'bg-fk-amber-soft text-fk-amber',
  danger: 'bg-fk-danger-soft text-fk-danger',
  primary: 'bg-fk-primary-soft text-fk-primary-text',
} as const

export function IconTile({
  tone,
  children,
  className,
}: {
  tone: keyof typeof iconTileTones
  children: ReactNode
  className?: string
}) {
  return (
    <span
      className={cn(
        'grid size-9 shrink-0 place-items-center rounded-md',
        iconTileTones[tone],
        className
      )}
    >
      {children}
    </span>
  )
}
