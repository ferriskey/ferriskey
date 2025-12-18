import { Card, CardContent } from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'
import { LucideIcon, TrendingUp, TrendingDown } from 'lucide-react'
import { ReactNode } from 'react'

interface StatisticsCardProps {
  title: string
  value: number | string
  description?: string | ReactNode
  icon?: LucideIcon
  isLoading?: boolean
  chart?: ReactNode
  trend?: {
    value: number
    direction: 'up' | 'down'
  }
  comparedTo?: string
}

export default function StatisticsCard({
  title,
  value,
  description,
  isLoading = false,
  chart,
  trend,
  comparedTo = 'compared to last week',
}: StatisticsCardProps) {
  if (isLoading) {
    return (
      <Card className='overflow-hidden'>
        <CardContent className='p-6'>
          <div className='space-y-3'>
            <Skeleton className='h-4 w-32' />
            <div className='flex items-end justify-between'>
              <div className='space-y-2'>
                <Skeleton className='h-10 w-24' />
                <Skeleton className='h-3 w-36' />
              </div>
              <Skeleton className='h-16 w-32' />
            </div>
          </div>
        </CardContent>
      </Card>
    )
  }

  const trendColor = trend?.direction === 'up' ? 'text-emerald-600' : 'text-red-600'
  const TrendIcon = trend?.direction === 'up' ? TrendingUp : TrendingDown

  return (
    <Card className='overflow-hidden hover:shadow-lg transition-shadow duration-200'>
      <CardContent className='p-6'>
        <div className='space-y-3'>
          {/* Title */}
          <h3 className='text-sm font-medium text-muted-foreground'>{title}</h3>

          {/* Value and Chart Row */}
          <div className='flex items-end justify-between gap-4'>
            <div className='flex-1'>
              {/* Main Value */}
              <div className='flex items-baseline gap-2 mb-1'>
                <span className='text-4xl font-bold tracking-tight'>{value}</span>
                {trend && (
                  <span className={`flex items-center gap-1 text-sm font-semibold ${trendColor}`}>
                    <TrendIcon className='h-3.5 w-3.5' />
                    {Math.abs(trend.value)}%
                  </span>
                )}
              </div>

              {/* Description / Compared To */}
              {(description || comparedTo) && (
                <p className='text-xs text-muted-foreground'>
                  {description || comparedTo}
                </p>
              )}
            </div>

            {/* Inline Chart */}
            {chart && (
              <div className='flex-shrink-0 w-32 h-16'>
                {chart}
              </div>
            )}
          </div>
        </div>
      </CardContent>
    </Card>
  )
}
