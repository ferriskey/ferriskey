import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'
import { LucideIcon } from 'lucide-react'

interface StatisticsCardProps {
  title: string
  value: number
  description: string
  icon: LucideIcon
  isLoading?: boolean
}

export default function StatisticsCard({
  title,
  value,
  description,
  icon: Icon,
  isLoading = false,
}: StatisticsCardProps) {
  return (
    <Card>
      <CardHeader className='flex flex-row items-center justify-between space-y-0 pb-2'>
        <CardTitle className='text-sm font-medium'>{title}</CardTitle>
        <Icon className='h-4 w-4 text-muted-foreground' />
      </CardHeader>
      <CardContent>
        {isLoading ? (
          <>
            <Skeleton className='h-8 w-16 mb-1' />
            <Skeleton className='h-4 w-24' />
          </>
        ) : (
          <>
            <div className='text-2xl font-bold'>{value}</div>
            <p className='text-xs text-muted-foreground'>{description}</p>
          </>
        )}
      </CardContent>
    </Card>
  )
}
