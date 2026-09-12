import type { ReactNode } from 'react'
import { Link } from 'react-router-dom'
import { cn } from '@/lib/utils'

export interface TabItem {
  key: string
  label: string
  href?: string
  count?: number
  warn?: boolean
}

interface TabsProps {
  tabs: readonly TabItem[]
  value: string
  onChange?: (key: string) => void
  children?: ReactNode
  className?: string
}

export function PageTabs({
  tabs,
  value,
  onChange,
  children,
  className,
}: TabsProps) {
  return (
    <div className={className}>
      <div role='tablist' className='flex gap-6 overflow-x-auto scrollbar-none border-b border-fk-line'>
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
                    active ? 'text-fk-primary-text' : 'text-neutral-400 dark:text-neutral-500'
                  )}
                >
                  {tab.count}
                </span>
              )}
            </>
          )

          const shape = cn(
            'mb-0 flex cursor-pointer items-center border-b-2 pb-2 text-[13px] transition-colors',
            active
              ? 'border-fk-brand font-medium text-neutral-900 dark:text-neutral-100'
              : 'border-transparent text-neutral-500 hover:text-neutral-900 dark:text-neutral-400 dark:hover:text-neutral-100'
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
