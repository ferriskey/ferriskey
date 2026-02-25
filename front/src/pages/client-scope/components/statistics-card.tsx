import { Card, CardContent } from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'
import { ReactNode } from 'react'

interface StatisticsCardProps {
  title: string
  value: number | string
  description?: string | ReactNode
  isLoading?: boolean
}

export default function StatisticsCard({
  title,
  value,
  description,
  isLoading = false,
}: StatisticsCardProps) {
  if (isLoading) {
    return (
      <Card className='overflow-hidden'>
        <CardContent className='p-6'>
          <div className='space-y-3'>
            <Skeleton className='h-4 w-32' />
            <Skeleton className='h-10 w-24' />
            <Skeleton className='h-3 w-36' />
          </div>
        </CardContent>
      </Card>
    )
  }

  return (
    <Card className='overflow-hidden hover:shadow-lg transition-shadow duration-200'>
      <CardContent className='p-6'>
        <div className='space-y-3'>
          <h3 className='text-sm font-medium text-muted-foreground'>{title}</h3>
          <span className='text-4xl font-bold tracking-tight'>{value}</span>
          {description && (
            <div className='text-xs text-muted-foreground'>{description}</div>
          )}
        </div>
      </CardContent>
    </Card>
  )
}
