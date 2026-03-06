import { Schemas } from '@/api/api.client'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'
import { Activity, CheckCircle, Clock, XCircle } from 'lucide-react'

import FlowStats = Schemas.FlowStats

interface StatsCardsProps {
  stats: FlowStats | null
  isLoading: boolean
}

export function StatsCards({ stats, isLoading }: StatsCardsProps) {
  if (isLoading) {
    return (
      <div className='grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4'>
        {Array.from({ length: 4 }).map((_, i) => (
          <Card key={`skeleton-stat-${i}`}>
            <CardHeader className='pb-2'>
              <Skeleton className='h-4 w-24' />
            </CardHeader>
            <CardContent>
              <Skeleton className='h-8 w-16' />
            </CardContent>
          </Card>
        ))}
      </div>
    )
  }

  const cards = [
    {
      title: 'Total Flows',
      value: stats?.total ?? 0,
      icon: Activity,
      color: 'text-blue-500',
      bg: 'bg-blue-500/10',
    },
    {
      title: 'Successful',
      value: stats?.success_count ?? 0,
      icon: CheckCircle,
      color: 'text-emerald-500',
      bg: 'bg-emerald-500/10',
    },
    {
      title: 'Failed',
      value: stats?.failure_count ?? 0,
      icon: XCircle,
      color: 'text-red-500',
      bg: 'bg-red-500/10',
    },
    {
      title: 'Avg Duration',
      value: stats?.avg_duration_ms != null ? `${Math.round(stats.avg_duration_ms)}ms` : '—',
      icon: Clock,
      color: 'text-amber-500',
      bg: 'bg-amber-500/10',
    },
  ]

  return (
    <div className='grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4'>
      {cards.map((card) => (
        <Card key={card.title}>
          <CardHeader className='flex flex-row items-center justify-between pb-2'>
            <CardTitle className='text-sm font-medium text-muted-foreground'>{card.title}</CardTitle>
            <div className={`h-8 w-8 rounded-lg ${card.bg} flex items-center justify-center`}>
              <card.icon className={`h-4 w-4 ${card.color}`} />
            </div>
          </CardHeader>
          <CardContent>
            <div className='text-2xl font-bold'>{card.value}</div>
          </CardContent>
        </Card>
      ))}
    </div>
  )
}
