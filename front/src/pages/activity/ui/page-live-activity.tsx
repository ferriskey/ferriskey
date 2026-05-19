import { Schemas } from '@/api/api.client'
import { Card, CardContent, CardHeader } from '@/components/ui/card'
import {
  ChartConfig,
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent,
} from '@/components/ui/chart'
import { Skeleton } from '@/components/ui/skeleton'
import { ChevronDown } from 'lucide-react'
import { useMemo, useState } from 'react'
import { CartesianGrid, Line, LineChart, XAxis, YAxis } from 'recharts'

import DailyActivityStats = Schemas.DailyActivityStats

type Range = '7d' | '30d' | '90d'

const rangeLabels: Record<Range, string> = {
  '7d': 'Last 7 days',
  '30d': 'Last 30 days',
  '90d': 'Last 90 days',
}

const rangeWindows: Record<Range, { days: number; labelFmt: (d: Date) => string }> = {
  '7d': {
    days: 7,
    labelFmt: (d) => d.toLocaleDateString(undefined, { weekday: 'short', day: 'numeric' }),
  },
  '30d': {
    days: 30,
    labelFmt: (d) => d.toLocaleDateString(undefined, { month: 'short', day: 'numeric' }),
  },
  '90d': {
    days: 90,
    labelFmt: (d) => d.toLocaleDateString(undefined, { month: 'short', day: 'numeric' }),
  },
}

interface Props {
  dailyActivity: DailyActivityStats[]
  isLoading: boolean
}

interface RangeSelectProps {
  value: Range
  onChange: (r: Range) => void
}

function RangeSelect({ value, onChange }: RangeSelectProps) {
  return (
    <div className='relative inline-flex'>
      <select
        value={value}
        onChange={(e) => onChange(e.target.value as Range)}
        className='appearance-none rounded-md border border-border bg-background pl-3 pr-7 py-1 text-xs font-medium text-foreground hover:bg-muted transition-colors cursor-pointer outline-none focus:ring-1 focus:ring-primary/40'
      >
        {(Object.keys(rangeLabels) as Range[]).map((k) => (
          <option key={k} value={k}>
            {rangeLabels[k]}
          </option>
        ))}
      </select>
      <ChevronDown className='pointer-events-none absolute right-2 top-1/2 -translate-y-1/2 h-3 w-3 text-muted-foreground' />
    </div>
  )
}

function buildSeries({
  dailyActivity,
  range,
}: {
  dailyActivity: DailyActivityStats[]
  range: Range
}) {
  const { days, labelFmt } = rangeWindows[range]
  const visible = dailyActivity.slice(-days)
  const buckets = visible.map((day) => {
    const date = new Date(`${day.date}T00:00:00`)

    return {
      date: date.getTime(),
      label: labelFmt(date),
      signups: day.signups,
      logins: day.logins,
    }
  })

  const totalSignups = visible.reduce((sum, day) => sum + day.signups, 0)
  const totalLogins = visible.reduce((sum, day) => sum + day.logins, 0)

  return { buckets, totalSignups, totalLogins }
}

const chartConfig = {
  signups: {
    label: 'Sign-ups',
    color: 'oklch(0.696 0.17 162.48)',
  },
  logins: {
    label: 'Logins',
    color: 'oklch(0.6 0.18 250)',
  },
} satisfies ChartConfig

export default function PageLiveActivity({ dailyActivity, isLoading }: Props) {
  const [signupsRange, setSignupsRange] = useState<Range>('7d')
  const [loginsRange, setLoginsRange] = useState<Range>('7d')
  const [combinedRange, setCombinedRange] = useState<Range>('7d')

  const signupsSeries = useMemo(
    () => buildSeries({ dailyActivity, range: signupsRange }),
    [dailyActivity, signupsRange],
  )
  const loginsSeries = useMemo(
    () => buildSeries({ dailyActivity, range: loginsRange }),
    [dailyActivity, loginsRange],
  )
  const combinedSeries = useMemo(
    () => buildSeries({ dailyActivity, range: combinedRange }),
    [dailyActivity, combinedRange],
  )

  return (
    <div className='flex flex-col gap-8 p-8 md:p-12'>
      {/* Header */}
      <div className='flex flex-col gap-2 md:flex-row md:items-start md:justify-between'>
        <div>
          <h1 className='text-2xl font-medium tracking-tight'>Live activity</h1>
          <p className='text-sm text-muted-foreground mt-1'>
            Real-time view of customer sign-ups and logins.
          </p>
        </div>
        <p className='text-xs text-muted-foreground max-w-md'>
          You are currently viewing this realm&apos;s metrics. We&apos;re actively expanding our data
          views to provide more comprehensive metrics.
        </p>
      </div>

      {/* Big stat cards */}
      <div className='grid grid-cols-1 md:grid-cols-2 gap-6'>
        <BigStatCard
          label='Sign-ups'
          value={signupsSeries.totalSignups}
          color='text-emerald-500'
          range={signupsRange}
          onRangeChange={setSignupsRange}
          isLoading={isLoading}
        />
        <BigStatCard
          label='Logins'
          value={loginsSeries.totalLogins}
          color='text-blue-500'
          range={loginsRange}
          onRangeChange={setLoginsRange}
          isLoading={isLoading}
        />
      </div>

      {/* Combined activity chart */}
      <Card className='rounded-md'>
        <CardHeader className='flex flex-row items-start justify-between gap-4 pb-2 pt-6 px-6'>
          <div className='flex flex-col gap-3'>
            <div className='flex items-center gap-3'>
              <h2 className='text-base font-semibold'>Combined activity</h2>
              <RangeSelect value={combinedRange} onChange={setCombinedRange} />
            </div>
            <div className='flex items-center gap-6 text-sm'>
              <div className='flex flex-col'>
                <span className='text-xs text-muted-foreground'>Sign-ups</span>
                <span className='text-xl font-semibold text-emerald-500 tabular-nums'>
                  {isLoading ? '—' : combinedSeries.totalSignups}
                </span>
              </div>
              <div className='h-8 w-px bg-border' />
              <div className='flex flex-col'>
                <span className='text-xs text-muted-foreground'>Logins</span>
                <span className='text-xl font-semibold text-blue-500 tabular-nums'>
                  {isLoading ? '—' : combinedSeries.totalLogins}
                </span>
              </div>
            </div>
          </div>
        </CardHeader>
        <CardContent className='pb-6 px-6'>
          {isLoading ? (
            <Skeleton className='h-[280px] w-full rounded-md' />
          ) : (
            <ChartContainer config={chartConfig} className='h-[280px] w-full'>
              <LineChart data={combinedSeries.buckets} margin={{ top: 12, right: 16, left: 0, bottom: 0 }}>
                <CartesianGrid vertical={false} strokeDasharray='3 3' className='stroke-border/50' />
                <XAxis
                  dataKey='label'
                  tickLine={false}
                  axisLine={false}
                  tickMargin={10}
                  minTickGap={32}
                  tick={{ fontSize: 11 }}
                />
                <YAxis
                  tickLine={false}
                  axisLine={false}
                  width={28}
                  tick={{ fontSize: 11 }}
                  allowDecimals={false}
                />
                <ChartTooltip cursor={{ stroke: 'var(--border)', strokeWidth: 1 }} content={<ChartTooltipContent indicator='dot' />} />
                <Line
                  type='monotone'
                  dataKey='signups'
                  stroke='var(--color-signups)'
                  strokeWidth={2}
                  dot={{ r: 3, strokeWidth: 0, fill: 'var(--color-signups)' }}
                  activeDot={{ r: 5 }}
                />
                <Line
                  type='monotone'
                  dataKey='logins'
                  stroke='var(--color-logins)'
                  strokeWidth={2}
                  dot={{ r: 3, strokeWidth: 0, fill: 'var(--color-logins)' }}
                  activeDot={{ r: 5 }}
                />
              </LineChart>
            </ChartContainer>
          )}
          {/* Legend */}
          <div className='mt-4 flex items-center gap-4 text-xs'>
            <span className='inline-flex items-center gap-1.5'>
              <span className='h-2.5 w-2.5 rounded-sm bg-emerald-500' />
              <span className='text-muted-foreground'>Sign-ups</span>
            </span>
            <span className='inline-flex items-center gap-1.5'>
              <span className='h-2.5 w-2.5 rounded-sm bg-blue-500' />
              <span className='text-muted-foreground'>Logins</span>
            </span>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}

interface BigStatCardProps {
  label: string
  value: number
  color: string
  range: Range
  onRangeChange: (r: Range) => void
  isLoading: boolean
}

function BigStatCard({ label, value, color, range, onRangeChange, isLoading }: BigStatCardProps) {
  return (
    <Card className='rounded-md'>
      <CardHeader className='flex flex-row items-center justify-between pb-2 pt-6 px-6'>
        <span className='text-sm font-semibold'>{label}</span>
        <RangeSelect value={range} onChange={onRangeChange} />
      </CardHeader>
      <CardContent className='pb-6 px-6'>
        {isLoading ? (
          <Skeleton className='h-24 w-32 rounded-md' />
        ) : (
          <div className={`text-7xl font-light leading-none tabular-nums ${color}`}>{value}</div>
        )}
        <p className='text-xs text-muted-foreground mt-4'>
          {value === 0
            ? 'No significant change or no data available to determine trend'
            : `${value} ${label.toLowerCase()} in the ${rangeLabels[range].toLowerCase()}`}
        </p>
      </CardContent>
    </Card>
  )
}
