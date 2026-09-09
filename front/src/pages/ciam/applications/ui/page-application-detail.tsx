import type { ReactNode } from 'react'
import { ArrowLeft } from 'lucide-react'
import { Button } from '@/components/kit/button'
import { PageTabs, Pill, Squircle, type TabItem } from '@/components/kit'
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
  const container = cn('mx-auto', tokens.page.maxWidth, tokens.page.padding)

  const backButton = (
    <Button
      variant='ghost'
      size='sm'
      className='-ml-2 mb-2 text-neutral-500 dark:text-neutral-400'
      onClick={onBack}
    >
      <ArrowLeft className='size-3.5' />
      Applications
    </Button>
  )

  if (isLoading) {
    return (
      <div className={container}>
        <div className='h-4 w-24 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
        <div className='mt-4 flex items-center gap-3'>
          <div className='size-15 animate-pulse rounded-md bg-neutral-100 dark:bg-fk-raised' />
          <div className='space-y-2'>
            <div className='h-5 w-48 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
            <div className='h-4 w-32 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
          </div>
        </div>
      </div>
    )
  }

  if (!application) {
    return (
      <div className={container}>
        {backButton}
        <div className={cn(tokens.surface.panel, 'grid place-items-center px-6 py-16')}>
          <p className='text-sm font-medium text-neutral-700 dark:text-neutral-300'>
            Application not found
          </p>
          <p className='mt-1 max-w-sm text-center text-sm text-neutral-500 dark:text-neutral-400'>
            It may have been deleted, or it belongs to another realm.
          </p>
        </div>
      </div>
    )
  }

  const meta = applicationTypeMeta(inferApplicationType(application))
  const label = application.name || application.client_id

  return (
    <div className={container}>
      {backButton}

      <div className='flex flex-wrap items-start justify-between gap-4'>
        <div className='flex items-center gap-3'>
          <Squircle name={label} size='xl' />
          <div className='min-w-0'>
            <h1 className={tokens.header.title}>{label}</h1>
            <p className='font-mono-ui text-xs text-neutral-400 dark:text-neutral-500'>
              {application.client_id}
            </p>
            <div className='mt-1.5 flex flex-wrap items-center gap-2'>
              <Pill tone={meta.tone}>{meta.label}</Pill>
              <Pill tone={application.enabled ? 'success' : 'neutral'} mono>
                {application.enabled ? 'enabled' : 'disabled'}
              </Pill>
              {application.maintenance_enabled && <Pill tone='amber'>in maintenance</Pill>}
            </div>
          </div>
        </div>

        <dl className='shrink-0 text-right text-xs text-neutral-500 dark:text-neutral-400'>
          <dt className='sr-only'>Sign-in flow</dt>
          <dd>{meta.flow}</dd>
          <dt className='sr-only'>Created at</dt>
          <dd className='tnum'>Created {formatDate(application.created_at)}</dd>
        </dl>
      </div>

      <PageTabs tabs={tabs} value={tab} className='mt-5'>
        <div className={tokens.page.blockGap}>{children}</div>
      </PageTabs>
    </div>
  )
}
