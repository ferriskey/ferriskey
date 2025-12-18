import { useMemo } from 'react'
import { ChartContainer } from '@/components/ui/chart'
import { Area, AreaChart } from 'recharts'
import { Schemas } from '@/api/api.client'

import Client = Schemas.Client

interface ClientTrendChartProps {
  clients: Client[]
  days?: number
  useMockData?: boolean
  color?: string
}

const chartConfig = {
  count: {
    label: 'Clients',
    color: 'hsl(var(--chart-1))',
  },
}

const mockChartData = [
  { count: 3 },
  { count: 5 },
  { count: 2 },
  { count: 8 },
  { count: 6 },
  { count: 10 },
  { count: 7 },
]

export default function ClientTrendChart({
  clients,
  days = 7,
  useMockData = false,
  color = 'hsl(var(--chart-1))'
}: ClientTrendChartProps) {
  const chartData = useMemo(() => {
    if (useMockData) {
      return mockChartData
    }

    const dataPoints = []
    const today = new Date()

    for (let i = days - 1; i >= 0; i--) {
      const date = new Date(today)
      date.setDate(date.getDate() - i)
      date.setHours(0, 0, 0, 0)

      const nextDate = new Date(date)
      nextDate.setDate(nextDate.getDate() + 1)

      const count = clients.filter(client => {
        const createdAt = new Date(client.created_at)
        return createdAt >= date && createdAt < nextDate
      }).length

      dataPoints.push({ count })
    }

    return dataPoints
  }, [clients, days, useMockData])

  return (
    <ChartContainer config={chartConfig} className='h-full w-full'>
      <AreaChart
        accessibilityLayer
        data={chartData}
        margin={{
          left: 0,
          right: 0,
          top: 0,
          bottom: 2,
        }}
      >
        <defs>
          <linearGradient id='fillCount' x1='0' y1='0' x2='0' y2='1'>
            <stop offset='5%' stopColor={color} stopOpacity={0.3} />
            <stop offset='95%' stopColor={color} stopOpacity={0.05} />
          </linearGradient>
        </defs>
        <Area
          type='basis'
          dataKey='count'
          fill='url(#fillCount)'
          fillOpacity={1}
          stroke={color}
          strokeWidth={2}
          dot={false}
        />
      </AreaChart>
    </ChartContainer>
  )
}
