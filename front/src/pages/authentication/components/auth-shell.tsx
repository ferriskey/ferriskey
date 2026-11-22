import { useEffect, type ReactNode } from 'react'
import { KeyRound } from 'lucide-react'
import { useParams } from 'react-router'
import { cn } from '@/lib/utils'
import { BRAND_NAME } from '../constants'

export interface AuthShellProps {
  title: ReactNode
  description?: ReactNode
  asideContent?: ReactNode
  asideFooter?: ReactNode
  realmName?: string
  layout?: 'stacked' | 'split'
  contentClassName?: string
  children: ReactNode
}

export default function AuthShell({
  title,
  description,
  asideContent,
  asideFooter,
  realmName,
  layout = 'stacked',
  contentClassName,
  children,
}: AuthShellProps) {
  const { realm_name } = useParams()
  const realm = realmName ?? realm_name
  const split = layout === 'split'
  const realmChip = realm && (
    <span className='rounded-md border border-fk-line bg-fk-surface/70 px-1.5 py-0.5 font-mono text-[11px] text-fk-muted'>
      {realm}
    </span>
  )

  return (
    <AuthShellCenter>
      <div
        className={cn(
          'relative z-10 w-full overflow-hidden rounded-2xl border border-fk-line bg-fk-surface shadow-[0_28px_70px_-30px_rgb(0_0_0/0.35)]',
          split ? 'max-w-[880px]' : 'max-w-[480px]'
        )}
      >
        <div className={cn('grid', split && 'md:grid-cols-[minmax(0,5fr)_minmax(0,6fr)]')}>
          <aside
            className={cn(
              'relative flex flex-col justify-between overflow-hidden border-b border-fk-line bg-fk-sand',
              split ? 'gap-10 p-7 md:border-r md:border-b-0 md:p-9' : 'gap-4 p-5'
            )}
          >
            <KeyRound
              aria-hidden
              strokeWidth={1}
              className={cn(
                'pointer-events-none absolute -rotate-[18deg] text-fk-brand/[0.09]',
                split ? '-right-6 -bottom-8 size-44' : '-right-4 -bottom-6 size-28'
              )}
            />

            <div className={cn('relative', split ? 'space-y-8' : 'space-y-3')}>
              <div className='flex items-center gap-2.5'>
                <img src='/logo_ferriskey.png' alt={BRAND_NAME} className='size-5 object-contain' />
                <p className='font-mono text-[11px] font-semibold uppercase tracking-[0.22em] text-fk-primary-text'>
                  {BRAND_NAME}
                </p>
                {!split && <span className='ml-auto'>{realmChip}</span>}
              </div>

              <div className={split ? 'space-y-3' : 'space-y-1'}>
                <h1
                  className={cn(
                    'font-semibold tracking-tight text-fk-ink',
                    split ? 'text-2xl' : 'text-xl'
                  )}
                >
                  {title}
                </h1>
                {description && (
                  <p className={cn('text-sm text-fk-muted', split ? 'leading-7' : 'leading-6')}>
                    {description}
                  </p>
                )}
              </div>

              {asideContent}
            </div>

            {(split || asideFooter) && (
              <div className='relative flex flex-wrap items-center gap-x-3 gap-y-2'>
                {asideFooter}
                {split && realmChip}
              </div>
            )}
          </aside>

          <section className={cn('flex flex-col justify-center', contentClassName ?? (split ? 'p-6 md:p-9' : 'p-6 md:p-8'))}>
            {children}
          </section>
        </div>
      </div>
    </AuthShellCenter>
  )
}

export function AuthShellCenter({ children }: { children: ReactNode }) {
  useEffect(() => {
    document.documentElement.dataset.style = 'ferriskey'
    return () => {
      delete document.documentElement.dataset.style
    }
  }, [])

  return (
    <div className='relative isolate flex min-h-svh items-center justify-center overflow-hidden bg-fk-canvas px-4 py-10 text-fk-ink'>
      <div
        aria-hidden
        className='pointer-events-none absolute inset-0 -z-10 [background:radial-gradient(55%_45%_at_12%_0%,color-mix(in_oklab,var(--color-fk-brand)_16%,transparent),transparent_70%)]'
      />
      <div
        aria-hidden
        className='pointer-events-none absolute inset-0 -z-10 opacity-70 [background-image:radial-gradient(var(--color-fk-line)_1px,transparent_1px)] [background-size:22px_22px] [mask-image:radial-gradient(ellipse_at_center,black_25%,transparent_72%)]'
      />
      {children}
    </div>
  )
}
