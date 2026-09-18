import type { ReactNode } from 'react'
import { AlertTriangle, ArrowLeft, Clock, Loader, Monitor, User } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/kit/button'
import { PageShell, Pill, Section } from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import { Schemas } from '@/api/api.client'
import {
  failingStep,
  flowStatusTone,
  formatDateTime,
  formatDuration,
  orderedSteps,
  stepLabel,
} from '../flow-format'
import FlowSteps from './flow-steps'

import CompassFlow = Schemas.CompassFlow

export interface PageFlowDetailProps {
  flow?: CompassFlow
  isLoading: boolean
  isError: boolean
  onBack: () => void
}

function Meta({
  icon: Icon,
  label,
  children,
}: {
  icon: typeof Clock
  label: string
  children: ReactNode
}) {
  return (
    <div className={cn(tokens.surface.panel, 'px-3 py-2.5')}>
      <p className='flex items-center gap-1.5 text-[11px] text-neutral-500 dark:text-neutral-400'>
        <Icon className='size-3.5' strokeWidth={1.75} />
        {label}
      </p>
      <div className='mt-1 min-w-0 break-all text-xs text-neutral-900 dark:text-neutral-100'>{children}</div>
    </div>
  )
}

export default function PageFlowDetail({
  flow,
  isLoading,
  isError,
  onBack,
}: PageFlowDetailProps) {
  const { t } = useTranslation('compass')

  const backButton = (
    <Button
      variant='ghost'
      size='sm'
      className='-ml-2 mb-2 text-neutral-500 dark:text-neutral-400'
      onClick={onBack}
    >
      <ArrowLeft className='size-3.5' />
      {t('detail.back')}
    </Button>
  )

  if (isLoading) {
    return (
      <PageShell>
        <div className='h-4 w-24 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
        <div className='mt-4 space-y-2'>
          <div className='h-5 w-48 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
          <div className='h-4 w-64 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
        </div>
        <div className='mt-5 grid gap-3 sm:grid-cols-2 xl:grid-cols-4'>
          {Array.from({ length: 4 }).map((_, i) => (
            <div key={i} className='h-14 animate-pulse rounded-sm bg-neutral-100 dark:bg-fk-raised' />
          ))}
        </div>
      </PageShell>
    )
  }

  if (isError || !flow) {
    return (
      <PageShell>
        {backButton}
        <div className={cn(tokens.surface.panel, 'grid place-items-center px-6 py-16')}>
          <p className='text-sm font-medium text-neutral-700 dark:text-neutral-300'>
            {t('detail.not_found.title')}
          </p>
          <p className='mt-1 max-w-sm text-center text-sm text-neutral-500 dark:text-neutral-400'>
            {t('detail.not_found.detail')}
          </p>
        </div>
      </PageShell>
    )
  }

  const steps = orderedSteps(flow)
  const failing = failingStep(flow)
  const stepsTotal = steps.reduce((n, s) => n + (s.duration_ms ?? 0), 0)

  const originRows = [
    {
      key: 'grant_type',
      label: t('detail.origin.grant_type'),
      value: flow.grant_type,
    },
    {
      key: 'ip_address',
      label: t('detail.origin.ip_address'),
      value: flow.ip_address ?? t('detail.origin.not_recorded'),
    },
    {
      key: 'user_agent',
      label: t('detail.origin.user_agent'),
      value: flow.user_agent ?? t('detail.origin.not_recorded'),
    },
  ]

  return (
    <PageShell>
      {backButton}

      <div className='min-w-0'>
        <div className='flex flex-wrap items-center gap-2'>
          <h1 className={tokens.header.title}>{flow.grant_type}</h1>
          <Pill tone={flowStatusTone[flow.status]} mono>
            {flow.status === 'pending' && (
              <Loader className='size-3 animate-spin' strokeWidth={2.5} />
            )}
            {flow.status}
          </Pill>
          <Pill tone='neutral' mono>
            {formatDuration(flow.duration_ms)}
          </Pill>
        </div>
        <p className='mt-1 font-mono-ui text-xs text-neutral-400 dark:text-neutral-500'>{flow.id}</p>
      </div>

      {failing && (
        <div className='mt-4 flex items-start gap-2.5 rounded-sm border border-fk-danger-border bg-fk-danger-soft/40 px-4 py-3'>
          <AlertTriangle
            className='mt-0.5 size-4 shrink-0 text-fk-danger'
            strokeWidth={2}
          />
          <div className='min-w-0'>
            <p className='text-xs font-medium text-neutral-900 dark:text-neutral-100'>
              {t('detail.failure.title', { step: stepLabel(failing) })}
              {failing.error_code && (
                <span className='ml-1.5 font-mono-ui text-neutral-500 dark:text-neutral-400'>
                  {failing.error_code}
                </span>
              )}
            </p>
            {failing.error_message && (
              <p className='mt-0.5 text-xs text-neutral-600 dark:text-neutral-400'>{failing.error_message}</p>
            )}
          </div>
        </div>
      )}

      {flow.status === 'expired' && (
        <div className='mt-4 rounded-sm border border-fk-amber-border bg-fk-amber-soft/50 px-4 py-3 text-xs text-neutral-700 dark:text-neutral-300'>
          {t('detail.expired')}
        </div>
      )}

      <div className={cn('mt-5', tokens.page.blockGap)}>
        <div className='grid gap-3 sm:grid-cols-2 xl:grid-cols-4'>
          <Meta icon={Clock} label={t('detail.meta.started')}>
            {formatDateTime(flow.started_at)}
          </Meta>
          <Meta icon={Clock} label={t('detail.meta.completed')}>
            {flow.completed_at ? (
              formatDateTime(flow.completed_at)
            ) : (
              <span className='text-neutral-400 dark:text-neutral-500'>
                {flow.status === 'pending'
                  ? t('detail.meta.in_progress')
                  : t('detail.meta.never')}
              </span>
            )}
          </Meta>
          <Meta icon={Monitor} label={t('detail.meta.client')}>
            {flow.client_id ? (
              <span className='font-mono-ui'>{flow.client_id}</span>
            ) : (
              <span className='text-neutral-400 dark:text-neutral-500'>
                {t('detail.meta.unresolved')}
              </span>
            )}
          </Meta>
          <Meta icon={User} label={t('detail.meta.user')}>
            {flow.user_id ? (
              <span className='font-mono-ui'>{flow.user_id}</span>
            ) : (
              <span className='text-neutral-400 dark:text-neutral-500'>
                {t('detail.meta.never_identified')}
              </span>
            )}
          </Meta>
        </div>

        <Section
          title={t('detail.steps.title')}
          description={
            steps.length > 0
              ? t('detail.steps.description', {
                  count: steps.length,
                  duration: formatDuration(stepsTotal),
                })
              : t('detail.steps.empty')
          }
        >
          <FlowSteps steps={steps} pending={flow.status === 'pending'} />
        </Section>

        <Section title={t('detail.origin.title')}>
          <dl className={tokens.surface.divider}>
            {originRows.map(({ key, label, value }) => (
              <div
                key={key}
                className='grid gap-x-6 py-2.5 md:grid-cols-[minmax(0,12rem)_minmax(0,1fr)]'
              >
                <dt className='text-xs text-neutral-500 dark:text-neutral-400'>{label}</dt>
                <dd className='min-w-0 break-all font-mono-ui text-xs text-neutral-900 dark:text-neutral-100'>
                  {value}
                </dd>
              </div>
            ))}
          </dl>
        </Section>
      </div>
    </PageShell>
  )
}
