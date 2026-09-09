import { Area, AreaChart, CartesianGrid, ResponsiveContainer, Tooltip, XAxis, YAxis } from 'recharts'
import { Schemas } from '@/api/api.client'

import DailyActivityStats = Schemas.DailyActivityStats

export interface OverviewActivityChartProps {
  data: DailyActivityStats[]
}

const formatDay = (value: string) => {
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return value
  return date.toLocaleDateString('en-GB', { month: 'short', day: 'numeric' })
}

export default function OverviewActivityChart({ data }: OverviewActivityChartProps) {
  return (
    <ResponsiveContainer width='100%' height={168}>
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
        <CartesianGrid vertical={false} strokeDasharray='3 3' stroke='#f2f2f2' />
        <XAxis
          dataKey='date'
          tickFormatter={formatDay}
          tickLine={false}
          axisLine={false}
          minTickGap={28}
          tickMargin={8}
          tick={{ fontSize: 11, fill: '#a3a3a3' }}
        />
        <YAxis
          width={28}
          allowDecimals={false}
          tickLine={false}
          axisLine={false}
          tick={{ fontSize: 11, fill: '#a3a3a3' }}
        />
        <Tooltip
          labelFormatter={(label) => formatDay(String(label))}
          contentStyle={{
            border: '1px solid #e8e8e8',
            borderRadius: 2,
            fontSize: 12,
            padding: '6px 8px',
          }}
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
          dot={false}
        />
      </AreaChart>
    </ResponsiveContainer>
  )
}
