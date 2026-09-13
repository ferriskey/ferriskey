import {
  Area,
  AreaChart,
  CartesianGrid,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from 'recharts'
import { Schemas } from '@/api/api.client'

import DailyActivityStats = Schemas.DailyActivityStats

const toneHex = {
  info: '#2379ff',
  success: '#009764',
  amber: '#d97706',
  violet: '#8b5cf6',
  brand: '#e8431a',
  primary: '#c0602a',
} as const

export type ChartTone = keyof typeof toneHex

export function Sparkline({
  data,
  tone = 'info',
  height = 28,
}: {
  data: (number | null)[]
  tone?: ChartTone
  height?: number
}) {
  const points = data.map((v, i) => ({ i, v }))
  const color = toneHex[tone]
  const id = `spark-${tone}-${data.join('-')}`

  const values = data.filter((v): v is number => v !== null)
  const min = values.length > 0 ? Math.min(...values) : 0
  const max = values.length > 0 ? Math.max(...values) : 0
  const pad = max === min ? Math.max(Math.abs(max) * 0.5, 1) : (max - min) * 0.25

  return (
    <ResponsiveContainer width='100%' height={height}>
      <AreaChart data={points} margin={{ top: 3, right: 1, bottom: 3, left: 1 }}>
        <defs>
          <linearGradient id={id} x1='0' y1='0' x2='0' y2='1'>
            <stop offset='0%' stopColor={color} stopOpacity={0.28} />
            <stop offset='100%' stopColor={color} stopOpacity={0} />
          </linearGradient>
        </defs>
        <YAxis hide domain={[min - pad, max + pad]} />
        <Area
          type='monotone'
          dataKey='v'
          stroke={color}
          strokeWidth={2}
          fill={`url(#${id})`}
          isAnimationActive={false}
          connectNulls
          dot={false}
        />
      </AreaChart>
    </ResponsiveContainer>
  )
}

export interface ActivityChartProps {
  data: DailyActivityStats[]
  height?: number
}

const formatDay = (value: string) => {
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return value
  return date.toLocaleDateString('en-GB', { month: 'short', day: 'numeric' })
}

export function ActivityChart({ data, height = 168 }: ActivityChartProps) {
  return (
    <ResponsiveContainer width='100%' height={height}>
      <AreaChart data={data} margin={{ top: 6, right: 4, bottom: 0, left: 0 }}>
        <defs>
          <linearGradient id='fk-activity-logins' x1='0' y1='0' x2='0' y2='1'>
            <stop offset='0%' stopColor='#009764' stopOpacity={0.28} />
            <stop offset='100%' stopColor='#009764' stopOpacity={0} />
          </linearGradient>
          <linearGradient id='fk-activity-failures' x1='0' y1='0' x2='0' y2='1'>
            <stop offset='0%' stopColor='#dc2626' stopOpacity={0.22} />
            <stop offset='100%' stopColor='#dc2626' stopOpacity={0} />
          </linearGradient>
        </defs>
        <CartesianGrid vertical={false} strokeDasharray='3 3' stroke='var(--color-fk-line-soft)' />
        <XAxis
          dataKey='date'
          tickFormatter={formatDay}
          tickLine={false}
          axisLine={false}
          minTickGap={28}
          tickMargin={8}
          tick={{ fontSize: 11, fill: 'var(--color-fk-muted)' }}
        />
        <YAxis
          width={28}
          allowDecimals={false}
          tickLine={false}
          axisLine={false}
          tick={{ fontSize: 11, fill: 'var(--color-fk-muted)' }}
        />
        <Tooltip
          labelFormatter={(label) => formatDay(String(label))}
          contentStyle={{
            background: 'var(--color-fk-surface)',
            border: '1px solid var(--color-fk-line)',
            borderRadius: 2,
            fontSize: 12,
            padding: '6px 8px',
          }}
          labelStyle={{ color: 'var(--color-fk-ink)' }}
        />
        <Area
          type='monotone'
          dataKey='logins'
          name='Logins'
          stroke='#009764'
          strokeWidth={2}
          fill='url(#fk-activity-logins)'
          isAnimationActive={false}
          dot={false}
        />
        <Area
          type='monotone'
          dataKey='login_failures'
          name='Failures'
          stroke='#dc2626'
          strokeWidth={2}
          fill='url(#fk-activity-failures)'
          isAnimationActive={false}
          connectNulls
          dot={false}
        />
      </AreaChart>
    </ResponsiveContainer>
  )
}
