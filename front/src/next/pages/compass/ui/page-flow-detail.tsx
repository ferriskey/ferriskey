import type { ReactNode } from 'react'
import { AlertTriangle, ArrowLeft, Clock, Loader, Monitor, User } from 'lucide-react'
import { Button } from '@/components/kit/button'
import { Pill, Section } from '@/components/kit'
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
  const container = cn('mx-auto', tokens.page.maxWidth, tokens.page.padding)

  const backButton = (
    <Button
      variant='ghost'
      size='sm'
      className='-ml-2 mb-2 text-neutral-500 dark:text-neutral-400'
      onClick={onBack}
    >
      <ArrowLeft className='size-3.5' />
      Compass
    </Button>
  )

  if (isLoading) {
    return (
      <div className={container}>
        <div className='h-4 w-24 animate-pulse rounded bg-neutral-100 dark:bg-neutral-800' />
        <div className='mt-4 space-y-2'>
          <div className='h-5 w-48 animate-pulse rounded bg-neutral-100 dark:bg-neutral-800' />
          <div className='h-4 w-64 animate-pulse rounded bg-neutral-100 dark:bg-neutral-800' />
        </div>
        <div className='mt-5 grid gap-3 sm:grid-cols-2 xl:grid-cols-4'>
          {Array.from({ length: 4 }).map((_, i) => (
            <div key={i} className='h-14 animate-pulse rounded-sm bg-neutral-100 dark:bg-neutral-800' />
          ))}
        </div>
      </div>
    )
  }

  if (isError || !flow) {
    return (
      <div className={container}>
        {backButton}
        <div className={cn(tokens.surface.panel, 'grid place-items-center px-6 py-16')}>
          <p className='text-sm font-medium text-neutral-700 dark:text-neutral-300'>Execution not found</p>
          <p className='mt-1 max-w-sm text-center text-sm text-neutral-500 dark:text-neutral-400'>
            It may have been purged with the realm retention, or it belongs to another
            realm.
          </p>
        </div>
      </div>
    )
  }

  const steps = orderedSteps(flow)
  const failing = failingStep(flow)
  const stepsTotal = steps.reduce((n, s) => n + (s.duration_ms ?? 0), 0)

  return (
    <div className={container}>
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
              Failed at {stepLabel(failing)}
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
          This execution never came back. No step failed — the user simply did not
          return, so neither an end date nor a duration was ever recorded.
        </div>
      )}

      <div className={cn('mt-5', tokens.page.blockGap)}>
        <div className='grid gap-3 sm:grid-cols-2 xl:grid-cols-4'>
          <Meta icon={Clock} label='Started'>
            {formatDateTime(flow.started_at)}
          </Meta>
          <Meta icon={Clock} label='Completed'>
            {flow.completed_at ? (
              formatDateTime(flow.completed_at)
            ) : (
              <span className='text-neutral-400 dark:text-neutral-500'>
                {flow.status === 'pending' ? 'in progress' : 'never'}
              </span>
            )}
          </Meta>
          <Meta icon={Monitor} label='Client'>
            {flow.client_id ? (
              <span className='font-mono-ui'>{flow.client_id}</span>
            ) : (
              <span className='text-neutral-400 dark:text-neutral-500'>unresolved</span>
            )}
          </Meta>
          <Meta icon={User} label='User'>
            {flow.user_id ? (
              <span className='font-mono-ui'>{flow.user_id}</span>
            ) : (
              <span className='text-neutral-400 dark:text-neutral-500'>never identified</span>
            )}
          </Meta>
        </div>

        <Section
          title='Steps'
          description={
            steps.length > 0
              ? `${steps.length} step${steps.length > 1 ? 's' : ''} recorded — ${formatDuration(stepsTotal)} of cumulated processing, waiting on the user excluded.`
              : 'No step recorded for this execution.'
          }
        >
          <FlowSteps steps={steps} pending={flow.status === 'pending'} />
        </Section>

        <Section title='Origin'>
          <dl className={tokens.surface.divider}>
            {(
              [
                ['Grant type', flow.grant_type],
                ['IP address', flow.ip_address ?? 'not recorded'],
                ['User agent', flow.user_agent ?? 'not recorded'],
              ] as const
            ).map(([key, value]) => (
              <div
                key={key}
                className='grid gap-x-6 py-2.5 md:grid-cols-[minmax(0,12rem)_minmax(0,1fr)]'
              >
                <dt className='text-xs text-neutral-500 dark:text-neutral-400'>{key}</dt>
                <dd className='min-w-0 break-all font-mono-ui text-xs text-neutral-900 dark:text-neutral-100'>
                  {value}
                </dd>
              </div>
            ))}
          </dl>
        </Section>
      </div>
    </div>
  )
}
