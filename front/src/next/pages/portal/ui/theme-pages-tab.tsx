import { Link } from 'react-router-dom'
import { AlertTriangle, Check } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Pill, Section } from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import { PORTAL_PAGES, humanizeBlockType } from '../portal-pages'
import type { PortalPageStatus } from '../theme-validation'

export interface ThemePagesTabProps {
  statuses: PortalPageStatus[]
  pageHref: (pageType: string) => string
}

export default function ThemePagesTab({ statuses, pageHref }: ThemePagesTabProps) {
  const failures = statuses.filter((s) => s.missing.length > 0)

  return (
    <Section
      title='Pages'
      description={
        failures.length === 0
          ? 'Every page carries its required blocks: this theme can be activated.'
          : `${failures.length} page${failures.length > 1 ? 's' : ''} block${failures.length > 1 ? '' : 's'} activation. All of them are listed below — fixing one does not reveal a new one.`
      }
    >
      <ul className={tokens.surface.divider}>
        {PORTAL_PAGES.map((page) => {
          const status = statuses.find((s) => s.pageType === page.type)
          const missing = status?.missing ?? []
          const required = status?.required ?? []
          const present = status?.present ?? []

          return (
            <li key={page.type} className='flex items-start gap-3 py-3'>
              <span
                className={cn(
                  'mt-0.5 grid size-4 shrink-0 place-items-center rounded-full',
                  missing.length === 0
                    ? 'bg-fk-success-soft text-fk-success'
                    : 'bg-fk-amber-soft text-fk-amber'
                )}
              >
                {missing.length === 0 ? (
                  <Check className='size-2.5' strokeWidth={3} />
                ) : (
                  <AlertTriangle className='size-2.5' strokeWidth={3} />
                )}
              </span>

              <div className='min-w-0 flex-1'>
                <div className='flex flex-wrap items-center gap-2'>
                  <p className='text-[13px] font-medium text-neutral-900'>{page.label}</p>
                  <Pill tone='neutral' mono>
                    {page.type}
                  </Pill>
                  {required.length === 0 && <Pill tone='neutral'>no required block</Pill>}
                </div>
                <p className='mt-0.5 text-xs text-neutral-500'>{page.description}</p>

                {required.length > 0 && (
                  <div className='mt-1.5 flex flex-wrap gap-1.5'>
                    {required.map((block) => {
                      const on = present.includes(block)
                      return (
                        <span
                          key={block}
                          className={cn(
                            'font-mono-ui inline-flex items-center gap-1 rounded border px-1.5 py-0.5 text-xs leading-5',
                            on
                              ? 'border-fk-success-border bg-fk-success-soft text-fk-success'
                              : 'border-fk-amber-border bg-fk-amber-soft text-fk-amber'
                          )}
                          title={on ? undefined : `${humanizeBlockType(block)} is missing`}
                        >
                          {on ? (
                            <Check className='size-2.5' strokeWidth={3} />
                          ) : (
                            <AlertTriangle className='size-2.5' strokeWidth={3} />
                          )}
                          {block}
                        </span>
                      )
                    })}
                  </div>
                )}
              </div>

              <Button variant='ghost' size='sm' className='shrink-0 text-xs' asChild>
                <Link to={pageHref(page.type)}>Edit</Link>
              </Button>
            </li>
          )
        })}
      </ul>
    </Section>
  )
}
