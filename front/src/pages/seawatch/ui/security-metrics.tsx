import { Schemas } from '@/api/api.client'
import { Card, CardContent } from '@/components/ui/card'
import { Activity, AlertTriangle, Clock, Shield, User } from 'lucide-react'
import { useMemo } from 'react'

import SecurityEvent = Schemas.SecurityEvent

interface SecurityMetricsProps {
  events: SecurityEvent[]
  isLoading: boolean
}

const formatCount = (value: number, isLoading: boolean) => {
  if (isLoading) return '...'
  return value.toLocaleString()
}

const getLatestTimestamp = (events: SecurityEvent[]) => {
  if (!events.length) return 'No activity yet'
  const latest = events.reduce((acc, event) => {
    if (!acc) return event.timestamp
    return new Date(event.timestamp).getTime() > new Date(acc).getTime() ? event.timestamp : acc
  }, '')
  if (!latest) return 'No activity yet'
  return new Intl.DateTimeFormat(undefined, {
    dateStyle: 'short',
    timeStyle: 'short',
  }).format(new Date(latest))
}

export function SecurityMetrics({ events, isLoading }: SecurityMetricsProps) {
  const metrics = useMemo(() => {
    const total = events.length
    const failures = events.filter((event) => event.status === 'failure').length
    const successes = total - failures
    const uniqueActors = new Set(events.map((event) => event.actor_id ?? 'unknown')).size
    const successRate = total ? Math.round((successes / total) * 100) : 0
    const latest = getLatestTimestamp(events)
    return {
      total,
      failures,
      uniqueActors,
      successRate,
      latest,
    }
  }, [events])

  return (
    <div className='grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6'>
      <Card>
        <CardContent className='p-6'>
          <div className='flex items-center justify-between'>
            <div>
              <p className='text-sm font-medium text-muted-foreground'>
                Total Events
              </p>
              <p className='text-3xl font-bold text-foreground'>
                {formatCount(metrics.total, isLoading)}
              </p>
              <p className='text-xs text-muted-foreground flex items-center mt-1'>
                <Activity className='h-3 w-3 mr-1' />
                Latest signals across the realm
              </p>
            </div>
            <div className='h-8 w-8 rounded-full bg-primary/10 flex items-center justify-center'>
              <Shield className='h-4 w-4 text-primary' />
            </div>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardContent className='p-6'>
          <div className='flex items-center justify-between'>
            <div>
              <p className='text-sm font-medium text-muted-foreground'>
                Failed Events
              </p>
              <p className='text-3xl font-bold text-red-600'>
                {formatCount(metrics.failures, isLoading)}
              </p>
              <p className='text-xs text-muted-foreground flex items-center mt-1'>
                <AlertTriangle className='h-3 w-3 mr-1 text-red-500' />
                {metrics.successRate}% success rate
              </p>
            </div>
            <div className='h-8 w-8 rounded-full bg-red-500/10 flex items-center justify-center'>
              <AlertTriangle className='h-4 w-4 text-red-500' />
            </div>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardContent className='p-6'>
          <div className='flex items-center justify-between'>
            <div>
              <p className='text-sm font-medium text-muted-foreground'>
                Unique Actors
              </p>
              <p className='text-3xl font-bold text-foreground'>
                {formatCount(metrics.uniqueActors, isLoading)}
              </p>
              <p className='text-xs text-muted-foreground flex items-center mt-1'>
                <User className='h-3 w-3 mr-1' />
                Distinct actors observed
              </p>
            </div>
            <div className='h-8 w-8 rounded-full bg-blue-500/10 flex items-center justify-center'>
              <User className='h-4 w-4 text-blue-500' />
            </div>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardContent className='p-6'>
          <div className='flex items-center justify-between'>
            <div>
              <p className='text-sm font-medium text-muted-foreground'>
                Last Event
              </p>
              <p className='text-2xl font-semibold text-foreground'>
                {isLoading ? '...' : metrics.latest}
              </p>
              <p className='text-xs text-muted-foreground flex items-center mt-1'>
                <Clock className='h-3 w-3 mr-1' />
                Updated automatically
              </p>
            </div>
            <div className='h-8 w-8 rounded-full bg-emerald-500/10 flex items-center justify-center'>
              <Clock className='h-4 w-4 text-emerald-500' />
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
