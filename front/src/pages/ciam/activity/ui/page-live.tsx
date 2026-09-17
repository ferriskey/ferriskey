import { useMemo, useState } from 'react'
import { Trans, useTranslation } from 'react-i18next'
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
import { formatRelative, formatTimestamp } from '@/utils/format-date'
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

const RANGE_DAYS = [7, 30, 90] as const

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
  const { t } = useTranslation('console')
  const [range, setRange] = useState(String(RANGE_DAYS[0]))

  const days = Number(range)
  const visible = activity.slice(-days)
  const windowLabel = t('activity.window', { count: days })
  const overWindow = t('activity.over_window', { window: windowLabel })

  const ranges = useMemo<SegmentedItem[]>(
    () =>
      RANGE_DAYS.map((count) => ({
        key: String(count),
        label: t('activity.live.range', { count }),
      })),
    [t]
  )

  const signups = sum(visible, 'signups')
  const logins = sum(visible, 'logins')
  const loginFailures = sum(visible, 'login_failures')
  const uniqueUsers = visible.reduce((max, day) => Math.max(max, day.unique_login_users), 0)

  const measured = (series: number[]) => (visible.length > 0 ? series : undefined)

  const metrics: Metric[] = [
    {
      key: 'signups',
      label: t('activity.live.metrics.signups'),
      value: t('number', { value: signups }),
      hint: signups === 0 ? t('activity.nothing_recorded') : overWindow,
      series: measured(visible.map((day) => day.signups)),
      tone: 'violet',
    },
    {
      key: 'logins',
      label: t('activity.live.metrics.logins'),
      value: t('number', { value: logins }),
      hint: logins === 0 ? t('activity.nothing_recorded') : overWindow,
      series: measured(visible.map((day) => day.logins)),
      tone: 'success',
    },
    {
      key: 'failures',
      label: t('activity.live.metrics.failures'),
      value: t('number', { value: loginFailures }),
      hint: loginFailures === 0 ? t('activity.no_failure') : overWindow,
      series: measured(visible.map((day) => day.login_failures)),
      tone: 'brand',
    },
    {
      key: 'accounts',
      label: t('activity.live.metrics.accounts'),
      value: t('number', { value: uniqueUsers }),
      hint: t('activity.live.metrics.busiest', { window: windowLabel }),
      series: measured(visible.map((day) => day.unique_login_users)),
      tone: 'info',
    },
  ]

  const notices: Notice[] = [
    ...(!compassEnabled
      ? [
          {
            tone: 'note' as const,
            title: t('activity.live.notices.compass_off.title'),
            detail: t('activity.live.notices.compass_off.detail'),
          },
        ]
      : []),
    ...(isError
      ? [
          {
            tone: 'error' as const,
            title: t('activity.live.notices.error.title'),
            detail: t('activity.live.notices.error.detail'),
          },
        ]
      : []),
  ]

  const hasSeries = compassEnabled && visible.length > 1

  return (
    <ActivityPage
      title={t('activity.live.title')}
      description={t('activity.live.description', { window: windowLabel })}
      action={<Segmented items={ranges} value={range} onChange={setRange} />}
    >
      <NoticeList notices={notices} />

      <MetricsBand metrics={metrics} />

      {hasSeries && (
        <>
          <Section
            title={t('activity.live.logins.title')}
            description={t('activity.live.logins.description', { window: windowLabel })}
            contained={false}
            action={
              <div className='flex items-center gap-3 text-[11px] text-neutral-500 dark:text-neutral-400'>
                <span className='inline-flex items-center gap-1'>
                  <span className='size-1.5 rounded-full bg-fk-success' />
                  <Trans
                    i18nKey='console:activity.live.logins.legend_logins'
                    values={{ total: logins }}
                    components={{ num: <span className='tnum' /> }}
                  />
                </span>
                <span className='inline-flex items-center gap-1'>
                  <span className='size-1.5 rounded-full bg-fk-danger' />
                  <Trans
                    i18nKey='console:activity.live.logins.legend_failures'
                    values={{ total: loginFailures }}
                    components={{ num: <span className='tnum' /> }}
                  />
                </span>
              </div>
            }
          >
            <div className={cn(tokens.surface.panel, 'px-2 py-2')}>
              <ActivityChart data={visible} height={168} />
            </div>
          </Section>

          <Section
            title={t('activity.live.signups.title')}
            description={t('activity.live.signups.description', { window: windowLabel })}
            contained={false}
            action={
              <span className='text-[11px] text-neutral-500 dark:text-neutral-400'>
                <Trans
                  i18nKey='console:activity.live.signups.legend'
                  values={{ total: signups }}
                  components={{ num: <span className='tnum' /> }}
                />
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
          title={t('activity.live.flows.title')}
          description={t('activity.live.flows.description')}
          contained={false}
        >
          <div className={cn(tokens.surface.panel, tokens.surface.divider)}>
            {isLoading && flows.length === 0 && (
              <div className='px-3 py-6 text-center text-sm text-neutral-500 dark:text-neutral-400'>
                {t('activity.live.flows.loading')}
              </div>
            )}
            {!isLoading && flows.length === 0 && (
              <div className='px-3 py-6 text-center text-sm text-neutral-500 dark:text-neutral-400'>
                {t('activity.live.flows.empty')}
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
                    t('activity.live.flows.unidentified')}
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
