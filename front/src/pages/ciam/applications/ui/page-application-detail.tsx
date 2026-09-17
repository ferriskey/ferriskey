import type { ReactNode } from 'react'
import { useTranslation } from 'react-i18next'
import { ArrowLeft } from 'lucide-react'
import { Button } from '@/components/kit/button'
import { DetailHeader, PageShell, PageTabs, Pill, Squircle, type TabItem } from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import { formatDate } from '@/utils/format-date'
import { Schemas } from '@/api/api.client'
import { applicationTypeMeta, inferApplicationType } from '../application-types'

import Client = Schemas.Client

export interface PageApplicationDetailProps {
  application?: Client
  isLoading: boolean
  tab: string
  tabs: TabItem[]
  onBack: () => void
  children: ReactNode
}

export default function PageApplicationDetail({
  application,
  isLoading,
  tab,
  tabs,
  onBack,
  children,
}: PageApplicationDetailProps) {
  const { t } = useTranslation('console')

  const backButton = (
    <Button
      variant='ghost'
      size='sm'
      className='-ml-2 mb-2 text-neutral-500 dark:text-neutral-400'
      onClick={onBack}
    >
      <ArrowLeft className='size-3.5' />
      {t('applications.detail.back')}
    </Button>
  )

  if (isLoading) {
    return (
      <PageShell>
        <div className='h-4 w-24 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
        <div className='mt-4 flex items-center gap-3'>
          <div className='size-15 animate-pulse rounded-md bg-neutral-100 dark:bg-fk-raised' />
          <div className='space-y-2'>
            <div className='h-5 w-48 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
            <div className='h-4 w-32 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
          </div>
        </div>
      </PageShell>
    )
  }

  if (!application) {
    return (
      <PageShell>
        {backButton}
        <div className={cn(tokens.surface.panel, 'grid place-items-center px-6 py-16')}>
          <p className='text-sm font-medium text-neutral-700 dark:text-neutral-300'>
            {t('applications.detail.not_found.title')}
          </p>
          <p className='mt-1 max-w-sm text-center text-sm text-neutral-500 dark:text-neutral-400'>
            {t('applications.detail.not_found.description')}
          </p>
        </div>
      </PageShell>
    )
  }

  const meta = applicationTypeMeta(inferApplicationType(application), t)
  const label = application.name || application.client_id

  return (
    <PageShell>
      <DetailHeader
        onBack={onBack}
        backLabel={t('applications.detail.back')}
        icon={<Squircle name={label} size='xl' />}
        title={label}
        caption={
          <p className='font-mono-ui text-xs text-neutral-400 dark:text-neutral-500'>
            {application.client_id}
          </p>
        }
        pills={
          <>
            <Pill tone={meta.tone}>{meta.label}</Pill>
            <Pill tone={application.enabled ? 'success' : 'neutral'} mono>
              {application.enabled
                ? t('applications.detail.state.enabled')
                : t('applications.detail.state.disabled')}
            </Pill>
            {application.maintenance_enabled && (
              <Pill tone='amber'>{t('applications.detail.state.maintenance')}</Pill>
            )}
          </>
        }
        meta={
          <dl className='shrink-0 text-right text-xs text-neutral-500 dark:text-neutral-400'>
            <dt className='sr-only'>{t('applications.detail.meta.flow_label')}</dt>
            <dd>{meta.flow}</dd>
            <dt className='sr-only'>{t('applications.detail.meta.created_label')}</dt>
            <dd className='tnum'>
              {t('applications.detail.meta.created', {
                date: formatDate(application.created_at),
              })}
            </dd>
          </dl>
        }
      />

      <PageTabs tabs={tabs} value={tab} className='mt-5'>
        <div className={tokens.page.blockGap}>{children}</div>
      </PageTabs>
    </PageShell>
  )
}
