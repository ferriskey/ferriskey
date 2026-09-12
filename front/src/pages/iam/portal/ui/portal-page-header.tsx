import type { ReactNode } from 'react'
import { PageTabs } from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import { usePortalUrls } from '../use-portal-urls'

export interface PortalPageHeaderProps {
  tab: 'themes' | 'layouts'
  actions?: ReactNode
}

export function PortalPageHeader({ tab, actions }: PortalPageHeaderProps) {
  const portal = usePortalUrls()

  return (
    <>
      <div
        className={cn('flex flex-wrap items-start justify-between gap-3', tokens.header.spacing)}
      >
        <div className='min-w-0'>
          <h1 className={tokens.header.title}>Portal</h1>
          <p className='mt-0.5 text-sm text-neutral-500 dark:text-neutral-400'>
            Appearance of the public authentication pages of this realm.
          </p>
        </div>
        {actions && <div className='flex shrink-0 items-center gap-2'>{actions}</div>}
      </div>

      <PageTabs
        value={tab}
        tabs={[
          { key: 'themes', label: 'Themes', href: portal.themes() },
          { key: 'layouts', label: 'Layouts', href: portal.layouts() },
        ]}
      />
    </>
  )
}
