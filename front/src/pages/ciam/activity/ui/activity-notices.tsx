import type { ReactNode } from 'react'
import { AlertTriangle, CheckCircle2, Info } from 'lucide-react'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'

export type NoticeTone = 'error' | 'warn' | 'ok' | 'note'

export interface Notice {
  tone: NoticeTone
  title: string
  detail?: string
}

const noticeTones: Record<NoticeTone, string> = {
  error: 'border-fk-danger-border bg-fk-danger-soft/40 text-fk-danger',
  warn: 'border-fk-amber-border bg-fk-amber-soft/50 text-fk-amber',
  ok: 'border-fk-success-border bg-fk-success-soft/50 text-fk-success',
  note: 'border-fk-line bg-neutral-50 text-neutral-500 dark:bg-fk-surface dark:text-neutral-400',
}

export function NoticeList({ notices }: { notices: Notice[] }) {
  if (notices.length === 0) return null

  return (
    <ul className='space-y-1'>
      {notices.map((notice) => (
        <li
          key={notice.title}
          className={cn(
            'flex items-center gap-2 rounded-sm border px-2.5 py-1.5 text-[13px]',
            noticeTones[notice.tone]
          )}
        >
          {notice.tone === 'ok' ? (
            <CheckCircle2 className='size-3.5 shrink-0' strokeWidth={2} />
          ) : notice.tone === 'note' ? (
            <Info className='size-3.5 shrink-0' strokeWidth={2} />
          ) : (
            <AlertTriangle className='size-3.5 shrink-0' strokeWidth={2} />
          )}
          <span className='shrink-0 font-medium text-neutral-900 dark:text-neutral-100'>
            {notice.title}
          </span>
          {notice.detail && (
            <span className='min-w-0 truncate text-neutral-500 dark:text-neutral-400'>
              {notice.detail}
            </span>
          )}
        </li>
      ))}
    </ul>
  )
}

export function ActivityPage({
  title,
  description,
  action,
  children,
}: {
  title: string
  description: string
  action?: ReactNode
  children: ReactNode
}) {
  return (
    <div className={cn('mx-auto', tokens.page.maxWidth, tokens.page.padding)}>
      <div
        className={cn(
          'flex flex-wrap items-start justify-between gap-3',
          tokens.header.spacing
        )}
      >
        <div className='min-w-0'>
          <h1 className={tokens.header.title}>{title}</h1>
          <p className='mt-0.5 text-sm text-neutral-500 dark:text-neutral-400'>
            {description}
          </p>
        </div>
        {action && <div className='shrink-0'>{action}</div>}
      </div>

      <div className={tokens.page.sectionGap}>{children}</div>
    </div>
  )
}
