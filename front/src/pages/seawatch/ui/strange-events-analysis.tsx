import { Schemas } from '@/api/api.client'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { formatSnakeCaseToTitleCase } from '@/utils'
import { AlertTriangle, Shield, TrendingUp } from 'lucide-react'
import { useMemo } from 'react'

import SecurityEvent = Schemas.SecurityEvent

export interface StrangeEventsAnalysisProps {
  events: SecurityEvent[]
}

const getTopEventTypes = (events: SecurityEvent[]) => {
  const total = events.length
  const counts = events.reduce((acc, event) => {
    acc[event.event_type] = (acc[event.event_type] ?? 0) + 1
    return acc
  }, {} as Record<string, number>)

  return Object.entries(counts)
    .map(([type, count]) => ({
      type,
      count,
      percentage: total ? Math.round((count / total) * 100) : 0,
    }))
    .sort((a, b) => b.count - a.count)
    .slice(0, 5)
}

export function StrangeEventsAnalysis({ events }: StrangeEventsAnalysisProps) {
  const breakdown = useMemo(() => getTopEventTypes(events), [events])
  const failureCount = events.filter((event) => event.status === 'failure').length

  return (
    <Card>
      <CardHeader className='pb-4'>
        <div className='flex items-center justify-between'>
          <CardTitle className='flex items-center gap-2'>
            <Shield className='h-4 w-4 text-primary' />
            Event Breakdown
          </CardTitle>
          <Badge variant={failureCount > 0 ? 'destructive' : 'secondary'}>
            {failureCount} failures
          </Badge>
        </div>
      </CardHeader>
      <CardContent>
        <div className='space-y-4'>
          {breakdown.length === 0 && (
            <div className='rounded-lg border border-dashed border-muted p-6 text-center text-sm text-muted-foreground'>
              No events to analyze yet.
            </div>
          )}
          {breakdown.map((entry) => (
            <div key={entry.type} className='space-y-2'>
              <div className='flex items-center justify-between text-sm'>
                <span className='font-medium'>{formatSnakeCaseToTitleCase(entry.type)}</span>
                <span className='text-muted-foreground'>{entry.count}</span>
              </div>
              <div className='h-2 w-full rounded-full bg-muted'>
                <div
                  className='h-2 rounded-full bg-primary/70'
                  style={{ width: `${entry.percentage}%` }}
                />
              </div>
            </div>
          ))}
          {breakdown.length > 0 && (
            <div className='flex items-center gap-2 text-xs text-muted-foreground'>
              <AlertTriangle className='h-3 w-3 text-amber-500' />
              <span>Focus on the top 5 event types to reduce noise.</span>
              <TrendingUp className='h-3 w-3 text-emerald-500' />
            </div>
          )}
        </div>
      </CardContent>
    </Card>
  )
}
