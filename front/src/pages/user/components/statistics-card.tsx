import { Card, CardContent } from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'
import { cn } from '@/lib/utils'
import { LucideIcon } from 'lucide-react'
import { ReactNode } from 'react'

interface StatisticsCardProps {
  title: string
  value: number | string
  description: string | ReactNode
  icon: LucideIcon
  isLoading?: boolean
  descriptionClassName?: string
}

export default function StatisticsCard({
  title,
  value,
  description,
  icon: Icon,
  isLoading = false,
  descriptionClassName,
}: StatisticsCardProps) {
  if (isLoading) {
    return (
      <Card>
        <CardContent>
          <div className='flex items-center justify-between'>
            <Skeleton className='h-4 w-24' />
            <Skeleton className='h-9 w-9 rounded-lg' />
          </div>
          <div className='mt-4'>
            <Skeleton className='h-10 w-20' />
            <Skeleton className='h-4 w-32 mt-2' />
          </div>
        </CardContent>
      </Card>
    )
  }

  return (
    <Card className='hover:shadow-md transition-shadow'>
      <CardContent>
        <div className='flex items-center justify-between'>
          <div className='text-sm font-medium text-muted-foreground'>{title}</div>
          <div className='rounded-lg bg-muted p-2'>
            <Icon className='h-5 w-5 text-muted-foreground' />
          </div>
        </div>
        <div className='mt-4'>
          <div className='text-4xl font-bold'>{value}</div>
          <div className={cn('text-sm mt-2', descriptionClassName || 'text-muted-foreground')}>
            {description}
          </div>
        </div>
      </CardContent>
    </Card>
  )
}
