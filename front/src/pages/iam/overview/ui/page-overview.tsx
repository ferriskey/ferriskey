import type { ComponentType } from 'react'
import { Link } from 'react-router'
import { AlertTriangle, ArrowRight, Check, CheckCircle2 } from 'lucide-react'
import { MetricsBand, PageShell, Section, type Metric } from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import { Schemas } from '@/api/api.client'
import OverviewActivityChart from './overview-activity-chart'
import OverviewEventLog, { type OverviewEvent } from './overview-event-log'

import DailyActivityStats = Schemas.DailyActivityStats

export interface OverviewAlert {
  key: string
  tone: 'warn' | 'error' | 'ok'
  title: string
  detail?: string
  action?: string
  onAction?: () => void
}

export interface OverviewCapability {
  key: string
  label: string
  description: string
  enabled: boolean
}

export interface OverviewQuickLink {
  key: string
  label: string
  description: string
  href: string
  icon: ComponentType<{ className?: string; strokeWidth?: number }>
}

export interface PageOverviewProps {
  realmTitle: string
  greeting?: string
  isLoading: boolean
  metrics: Metric[]
  alerts: OverviewAlert[]
  capabilities: OverviewCapability[]
  activity: DailyActivityStats[]
  activityWindowDays: number
  activityEmptyLabel: string
  events: OverviewEvent[]
  eventsEmptyLabel: string
  eventsHref: string
  quickLinks: OverviewQuickLink[]
}

const alertTones = {
  ok: 'border-fk-success-border bg-fk-success-soft/50 text-fk-success',
  warn: 'border-fk-amber-border bg-fk-amber-soft/50 text-fk-amber',
  error: 'border-fk-danger-border bg-fk-danger-soft/40 text-fk-danger',
} as const

export default function PageOverview({
  realmTitle,
  greeting,
  isLoading,
  metrics,
  alerts,
  capabilities,
  activity,
  activityWindowDays,
  activityEmptyLabel,
  events,
  eventsEmptyLabel,
  eventsHref,
  quickLinks,
}: PageOverviewProps) {
  if (isLoading) {
    return (
      <PageShell>
        <div className='h-5 w-56 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
        <div className={cn('mt-4', tokens.page.sectionGap)}>
          <div className='h-14 animate-pulse rounded-sm bg-neutral-100 dark:bg-fk-raised' />
          <div className='grid gap-3 xl:grid-cols-[minmax(0,2fr)_minmax(0,1fr)]'>
            <div className='h-56 animate-pulse rounded-sm bg-neutral-100 dark:bg-fk-raised' />
            <div className='h-56 animate-pulse rounded-sm bg-neutral-100 dark:bg-fk-raised' />
          </div>
        </div>
      </PageShell>
    )
  }

  const enabledCapabilities = capabilities.filter((c) => c.enabled).length
  const totalLogins = activity.reduce((n, day) => n + day.logins, 0)
  const totalFailures = activity.reduce((n, day) => n + day.login_failures, 0)
  const hasActivity = activity.length > 1 && totalLogins + totalFailures > 0

  return (
    <PageShell>
      <div className={cn('flex flex-wrap items-center gap-x-3 gap-y-2', tokens.header.spacing)}>
        <h1 className={tokens.header.title}>
          {greeting ? `Welcome back, ${greeting} 👋` : `${realmTitle} realm`}
        </h1>
      </div>

      <div className={tokens.page.sectionGap}>
        {alerts.length > 0 && (
          <ul className='space-y-1'>
            {alerts.map((alert) => (
              <li
                key={alert.key}
                className={cn(
                  'flex items-center gap-2 rounded-sm border px-2.5 py-1.5 text-[13px]',
                  alertTones[alert.tone]
                )}
              >
                {alert.tone === 'ok' ? (
                  <CheckCircle2 className='size-3.5 shrink-0' strokeWidth={2} />
                ) : (
                  <AlertTriangle className='size-3.5 shrink-0' strokeWidth={2} />
                )}
                <span className='shrink-0 font-medium text-neutral-900 dark:text-neutral-100'>{alert.title}</span>
                {alert.detail && (
                  <span className='min-w-0 truncate text-neutral-500 dark:text-neutral-400'>{alert.detail}</span>
                )}
                {alert.action && (
                  <button
                    type='button'
                    onClick={alert.onAction}
                    className='ml-auto shrink-0 cursor-pointer text-xs font-medium underline-offset-2 hover:underline'
                  >
                    {alert.action} →
                  </button>
                )}
              </li>
            ))}
          </ul>
        )}

        <MetricsBand metrics={metrics} />

        <div className='grid gap-3 xl:grid-cols-[minmax(0,2fr)_minmax(0,1fr)]'>
          <Section
            title='Authentication activity'
            description={`Logins and failures recorded over the last ${activityWindowDays} days.`}
            contained={false}
            action={
              hasActivity ? (
                <div className='flex items-center gap-3 text-[11px] text-neutral-500 dark:text-neutral-400'>
                  <span className='inline-flex items-center gap-1'>
                    <span className='size-1.5 rounded-full bg-fk-success' />
                    <span className='tnum'>{totalLogins}</span> logins
                  </span>
                  <span className='inline-flex items-center gap-1'>
                    <span className='size-1.5 rounded-full bg-fk-danger' />
                    <span className='tnum'>{totalFailures}</span> failures
                  </span>
                </div>
              ) : undefined
            }
          >
            <div className={cn(tokens.surface.panel, 'px-2 py-2')}>
              {hasActivity ? (
                <OverviewActivityChart data={activity} />
              ) : (
                <div className='grid h-[168px] place-items-center px-6 text-center'>
                  <p className='text-sm text-neutral-500 dark:text-neutral-400'>{activityEmptyLabel}</p>
                </div>
              )}
            </div>
          </Section>

          <Section
            title='Capabilities'
            description='Authentication features this realm exposes to its accounts.'
            action={
              <span className='tnum text-[11px] text-neutral-400 dark:text-neutral-500'>
                {enabledCapabilities}/{capabilities.length}
              </span>
            }
          >
            {capabilities.map((capability) => (
              <div key={capability.key} className='flex items-center gap-2 py-1.5 text-[13px]'>
                <span
                  className={cn(
                    'grid size-3.5 shrink-0 place-items-center rounded-full',
                    capability.enabled
                      ? 'bg-fk-success-soft text-fk-success'
                      : 'bg-neutral-100 dark:bg-fk-raised'
                  )}
                >
                  {capability.enabled && <Check className='size-2' strokeWidth={3.5} />}
                </span>
                <span className='min-w-0 flex-1'>
                  <span
                    className={cn(
                      'block truncate',
                      capability.enabled
                        ? 'text-neutral-900 dark:text-neutral-100'
                        : 'text-neutral-500 dark:text-neutral-400'
                    )}
                  >
                    {capability.label}
                  </span>
                  <span className='block truncate text-[11px] text-neutral-400 dark:text-neutral-500'>
                    {capability.description}
                  </span>
                </span>
                <span className='shrink-0 text-[11px] uppercase tracking-wide text-neutral-400 dark:text-neutral-500'>
                  {capability.enabled ? 'on' : 'off'}
                </span>
              </div>
            ))}
          </Section>
        </div>

        <div className='grid gap-3 xl:grid-cols-[minmax(0,2fr)_minmax(0,1fr)]'>
          <Section
            title='Recent authentication flows'
            description='The last traces Compass recorded for this realm.'
            contained={false}
            action={
              <Link to={eventsHref} className='text-[11px] text-fk-primary-text hover:underline'>
                Compass →
              </Link>
            }
          >
            <OverviewEventLog events={events} emptyLabel={eventsEmptyLabel} />
          </Section>

          <Section
            title='Get started'
            description='Jump into a workspace of this realm.'
            contained={false}
          >
            <div className='grid gap-2 sm:grid-cols-2 xl:grid-cols-1'>
              {quickLinks.map((link) => (
                <Link
                  key={link.key}
                  to={link.href}
                  className={cn(
                    tokens.surface.panel,
                    'group flex items-center gap-3 px-3 py-2.5 transition-colors hover:border-fk-primary-border'
                  )}
                >
                  <span className='grid size-8 shrink-0 place-items-center rounded-md bg-fk-primary-soft text-fk-primary-text'>
                    <link.icon className='size-4' strokeWidth={1.75} />
                  </span>
                  <span className='min-w-0 flex-1'>
                    <span className='block truncate text-[13px] font-medium text-neutral-900 dark:text-neutral-100'>
                      {link.label}
                    </span>
                    <span className='block truncate text-[11px] text-neutral-500 dark:text-neutral-400'>
                      {link.description}
                    </span>
                  </span>
                  <ArrowRight className='size-3.5 shrink-0 text-neutral-300 dark:text-neutral-600 transition-transform group-hover:translate-x-0.5' />
                </Link>
              ))}
            </div>
          </Section>
        </div>
      </div>
    </PageShell>
  )
}
