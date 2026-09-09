import { useState } from 'react'
import {
  ActivityChart,
  MetricsBand,
  Pill,
  Section,
  Segmented,
  Sparkline,
} from '@/components/kit'
import type { Metric, SegmentedItem } from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import { Schemas } from '@/api/api.client'
import type { RealmDirectory } from '@/hooks/use-realm-directory'
import { formatRelative, formatTimestamp } from '@/shared/format-date'
import { ActivityPage, NoticeList, type Notice } from './activity-notices'

import DailyActivityStats = Schemas.DailyActivityStats
import CompassFlow = Schemas.CompassFlow
import FlowStatus = Schemas.FlowStatus

export interface PageLiveProps {
  activity: DailyActivityStats[]
  flows: CompassFlow[]
  compassEnabled: boolean
  isLoading: boolean
  isError: boolean
  directory: RealmDirectory
}

const ranges: SegmentedItem[] = [
  { key: '7', label: '7 days' },
  { key: '30', label: '30 days' },
  { key: '90', label: '90 days' },
]

const flowTones: Record<FlowStatus, 'success' | 'danger' | 'amber' | 'info'> = {
  success: 'success',
  failure: 'danger',
  expired: 'amber',
  pending: 'info',
}

const sum = (days: DailyActivityStats[], key: keyof DailyActivityStats) =>
  days.reduce((total, day) => total + (typeof day[key] === 'number' ? (day[key] as number) : 0), 0)

export default function PageLive({
  activity,
  flows,
  compassEnabled,
  isLoading,
  isError,
  directory,
}: PageLiveProps) {
  const [range, setRange] = useState('7')

  const days = Number(range)
  const visible = activity.slice(-days)
  const windowLabel = `last ${days} days`

  const signups = sum(visible, 'signups')
  const logins = sum(visible, 'logins')
  const loginFailures = sum(visible, 'login_failures')
  const uniqueUsers = visible.reduce((max, day) => Math.max(max, day.unique_login_users), 0)

  const measured = (series: number[]) => (visible.length > 0 ? series : undefined)

  const metrics: Metric[] = [
    {
      key: 'signups',
      label: 'Sign-ups',
      value: signups.toLocaleString(),
      hint: signups === 0 ? 'nothing recorded' : `over the ${windowLabel}`,
      series: measured(visible.map((day) => day.signups)),
      tone: 'violet',
    },
    {
      key: 'logins',
      label: 'Logins',
      value: logins.toLocaleString(),
      hint: logins === 0 ? 'nothing recorded' : `over the ${windowLabel}`,
      series: measured(visible.map((day) => day.logins)),
      tone: 'success',
    },
    {
      key: 'failures',
      label: 'Failed logins',
      value: loginFailures.toLocaleString(),
      hint: loginFailures === 0 ? 'no failure recorded' : `over the ${windowLabel}`,
      series: measured(visible.map((day) => day.login_failures)),
      tone: 'brand',
    },
    {
      key: 'accounts',
      label: 'Peak daily accounts',
      value: uniqueUsers.toLocaleString(),
      hint: `busiest day of the ${windowLabel}`,
      series: measured(visible.map((day) => day.unique_login_users)),
      tone: 'info',
    },
  ]

  const notices: Notice[] = [
    ...(!compassEnabled
      ? [
          {
            tone: 'note' as const,
            title: 'Compass tracing is off for this realm',
            detail:
              'Sign-ups and logins are counted from traced authentication flows. With tracing off nothing is recorded, so this page has nothing to measure.',
          },
        ]
      : []),
    ...(isError
      ? [
          {
            tone: 'error' as const,
            title: 'Activity unavailable',
            detail: 'We could not fetch the activity of this realm. Please try again later.',
          },
        ]
      : []),
  ]

  const hasSeries = compassEnabled && visible.length > 1

  return (
    <ActivityPage
      title='Live'
      description={`Sign-ups and logins recorded in this realm over the ${windowLabel}, counted from the authentication flows Compass traces.`}
      action={<Segmented items={ranges} value={range} onChange={setRange} />}
    >
      <NoticeList notices={notices} />

      <MetricsBand metrics={metrics} />

      {hasSeries && (
        <>
          <Section
            title='Logins'
            description={`Successful and failed sign-ins per day over the ${windowLabel}.`}
            contained={false}
            action={
              <div className='flex items-center gap-3 text-[11px] text-neutral-500 dark:text-neutral-400'>
                <span className='inline-flex items-center gap-1'>
                  <span className='size-1.5 rounded-full bg-fk-success' />
                  <span className='tnum'>{logins}</span> logins
                </span>
                <span className='inline-flex items-center gap-1'>
                  <span className='size-1.5 rounded-full bg-fk-danger' />
                  <span className='tnum'>{loginFailures}</span> failures
                </span>
              </div>
            }
          >
            <div className={cn(tokens.surface.panel, 'px-2 py-2')}>
              <ActivityChart data={visible} height={168} />
            </div>
          </Section>

          <Section
            title='Sign-ups'
            description={`New accounts created per day over the ${windowLabel}.`}
            contained={false}
            action={
              <span className='text-[11px] text-neutral-500 dark:text-neutral-400'>
                <span className='tnum'>{signups}</span> sign-ups
              </span>
            }
          >
            <div className={cn(tokens.surface.panel, 'px-3 py-3')}>
              <Sparkline
                data={visible.map((day) => day.signups)}
                tone='violet'
                height={110}
              />
              <div className='mt-1 flex justify-between text-[11px] text-neutral-400 dark:text-neutral-500'>
                <span>{visible[0]?.date}</span>
                <span>{visible[visible.length - 1]?.date}</span>
              </div>
            </div>
          </Section>
        </>
      )}

      {compassEnabled && (
        <Section
          title='Latest authentication attempts'
          description='The most recent traced flows, as of the moment this page was loaded. Nothing here refreshes on its own.'
          contained={false}
        >
          <div className={cn(tokens.surface.panel, tokens.surface.divider)}>
            {isLoading && flows.length === 0 && (
              <div className='px-3 py-6 text-center text-sm text-neutral-500 dark:text-neutral-400'>
                Loading…
              </div>
            )}
            {!isLoading && flows.length === 0 && (
              <div className='px-3 py-6 text-center text-sm text-neutral-500 dark:text-neutral-400'>
                No authentication flow has been traced yet. Each sign-in attempt adds a line
                here.
              </div>
            )}
            {flows.map((flow) => (
              <div
                key={flow.id}
                className='flex items-center gap-3 px-3 py-2 text-[13px]'
              >
                <Pill tone={flowTones[flow.status]} mono>
                  {flow.status}
                </Pill>
                <span className='min-w-0 flex-1 truncate'>
                  {directory.userLabel(flow.user_id) ??
                    flow.user_id ??
                    'no account identified'}
                </span>
                <span className='hidden min-w-0 flex-1 truncate font-mono-ui text-[11px] text-neutral-400 sm:block dark:text-neutral-500'>
                  {flow.grant_type}
                  {flow.ip_address ? ` · ${flow.ip_address}` : ''}
                </span>
                <span
                  className='tnum shrink-0 text-[11px] text-neutral-400 dark:text-neutral-500'
                  title={formatTimestamp(flow.started_at)}
                >
                  {formatRelative(flow.started_at)}
                </span>
              </div>
            ))}
          </div>
        </Section>
      )}
    </ActivityPage>
  )
}
