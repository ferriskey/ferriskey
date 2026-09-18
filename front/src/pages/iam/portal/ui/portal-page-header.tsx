import type { ReactNode } from 'react'
import { useTranslation } from 'react-i18next'
import { PageTabs } from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import { usePortalUrls } from '../use-portal-urls'

export const PORTAL_TAB_THEMES = 'themes'
export const PORTAL_TAB_LAYOUTS = 'layouts'

export interface PortalPageHeaderProps {
  tab: typeof PORTAL_TAB_THEMES | typeof PORTAL_TAB_LAYOUTS
  actions?: ReactNode
}

export function PortalPageHeader({ tab, actions }: PortalPageHeaderProps) {
  const { t } = useTranslation('portal')
  const portal = usePortalUrls()

  return (
    <>
      <div
        className={cn('flex flex-wrap items-start justify-between gap-3', tokens.header.spacing)}
      >
        <div className='min-w-0'>
          <h1 className={tokens.header.title}>{t('header.title')}</h1>
          <p className='mt-0.5 text-sm text-neutral-500 dark:text-neutral-400'>
            {t('header.description')}
          </p>
        </div>
        {actions && <div className='flex shrink-0 items-center gap-2'>{actions}</div>}
      </div>

      <PageTabs
        value={tab}
        tabs={[
          { key: PORTAL_TAB_THEMES, label: t('header.tabs.themes'), href: portal.themes() },
          { key: PORTAL_TAB_LAYOUTS, label: t('header.tabs.layouts'), href: portal.layouts() },
        ]}
      />
    </>
  )
}
