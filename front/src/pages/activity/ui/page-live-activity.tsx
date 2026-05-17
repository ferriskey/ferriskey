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

import CompassFlow = Schemas.CompassFlow
import User = Schemas.User

type Range = '24h' | '7d' | '30d'

const rangeLabels: Record<Range, string> = {
  '24h': 'Last 24 hours',
  '7d': 'Last 7 days',
  '30d': 'Last 30 days',
}

const rangeWindows: Record<Range, { ms: number; bucketMs: number; labelFmt: (d: Date) => string }> = {
  '24h': {
    ms: 24 * 60 * 60 * 1000,
    bucketMs: 60 * 60 * 1000,
    labelFmt: (d) =>
      d.toLocaleString(undefined, { weekday: 'short', hour: '2-digit', minute: '2-digit' }),
  },
  '7d': {
    ms: 7 * 24 * 60 * 60 * 1000,
    bucketMs: 24 * 60 * 60 * 1000,
    labelFmt: (d) => d.toLocaleDateString(undefined, { weekday: 'short', day: 'numeric' }),
  },
  '30d': {
    ms: 30 * 24 * 60 * 60 * 1000,
    bucketMs: 24 * 60 * 60 * 1000,
    labelFmt: (d) => d.toLocaleDateString(undefined, { month: 'short', day: 'numeric' }),
  },
}

interface Props {
  users: User[]
  flows: CompassFlow[]
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
  users,
  flows,
  range,
}: {
  users: User[]
  flows: CompassFlow[]
  range: Range
}) {
  const { ms, bucketMs, labelFmt } = rangeWindows[range]
  const now = Date.now()
  const start = now - ms

  const buckets: { date: number; label: string; signups: number; logins: number }[] = []
  const firstBucket = Math.floor(start / bucketMs) * bucketMs
  for (let t = firstBucket; t <= now; t += bucketMs) {
    buckets.push({ date: t, label: labelFmt(new Date(t)), signups: 0, logins: 0 })
  }

  const indexFor = (ts: number) => {
    if (ts < firstBucket || ts > now) return -1
    return Math.floor((ts - firstBucket) / bucketMs)
  }

  let totalSignups = 0
  for (const u of users) {
    const ts = new Date(u.created_at).getTime()
    if (Number.isNaN(ts) || ts < start) continue
    totalSignups += 1
    const i = indexFor(ts)
    if (i >= 0 && i < buckets.length) buckets[i].signups += 1
  }

  let totalLogins = 0
  for (const f of flows) {
    if (f.status !== 'success') continue
    const ts = new Date(f.started_at).getTime()
    if (Number.isNaN(ts) || ts < start) continue
    totalLogins += 1
    const i = indexFor(ts)
    if (i >= 0 && i < buckets.length) buckets[i].logins += 1
  }

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

export default function PageLiveActivity({ users, flows, isLoading }: Props) {
  const [signupsRange, setSignupsRange] = useState<Range>('24h')
  const [loginsRange, setLoginsRange] = useState<Range>('24h')
  const [combinedRange, setCombinedRange] = useState<Range>('24h')

  const signupsSeries = useMemo(
    () => buildSeries({ users, flows: [], range: signupsRange }),
    [users, signupsRange],
  )
  const loginsSeries = useMemo(
    () => buildSeries({ users: [], flows, range: loginsRange }),
    [flows, loginsRange],
  )
  const combinedSeries = useMemo(
    () => buildSeries({ users, flows, range: combinedRange }),
    [users, flows, combinedRange],
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
