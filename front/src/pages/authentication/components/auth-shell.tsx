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
  contentClassName?: string
  children: ReactNode
}

export default function AuthShell({
  title,
  description,
  asideContent,
  asideFooter,
  realmName,
  contentClassName,
  children,
}: AuthShellProps) {
  const { realm_name } = useParams()
  const realm = realmName ?? realm_name

  return (
    <AuthShellCenter>
      <div className='relative z-10 w-full max-w-[880px] overflow-hidden rounded-2xl border border-fk-line bg-fk-surface shadow-[0_28px_70px_-30px_rgb(0_0_0/0.35)]'>
        <div className='grid md:grid-cols-[minmax(0,5fr)_minmax(0,6fr)]'>
          <aside className='relative flex flex-col justify-between gap-10 overflow-hidden border-b border-fk-line bg-fk-sand p-7 md:border-r md:border-b-0 md:p-9'>
            <KeyRound
              aria-hidden
              strokeWidth={1}
              className='pointer-events-none absolute -right-6 -bottom-8 size-44 -rotate-[18deg] text-fk-brand/[0.09]'
            />

            <div className='relative space-y-8'>
              <div className='flex items-center gap-2.5'>
                <img src='/logo_ferriskey.png' alt={BRAND_NAME} className='size-5 object-contain' />
                <p className='font-mono text-[11px] font-semibold uppercase tracking-[0.22em] text-fk-primary-text'>
                  {BRAND_NAME}
                </p>
              </div>

              <div className='space-y-3'>
                <h1 className='text-2xl font-semibold tracking-tight text-fk-ink'>{title}</h1>
                {description && <p className='text-sm leading-7 text-fk-muted'>{description}</p>}
              </div>

              {asideContent}
            </div>

            <div className='relative flex flex-wrap items-center gap-x-3 gap-y-2'>
              {asideFooter}
              {realm && (
                <span className='rounded-md border border-fk-line bg-fk-surface/70 px-1.5 py-0.5 font-mono text-[11px] text-fk-muted'>
                  {realm}
                </span>
              )}
            </div>
          </aside>

          <section className={cn('flex flex-col justify-center', contentClassName ?? 'p-6 md:p-9')}>
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
