import { Schemas } from '@/api/api.client'
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Heading } from '@/components/ui/heading'
import { Input } from '@/components/ui/input'
import { Skeleton } from '@/components/ui/skeleton'
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { formatSnakeCaseToTitleCase } from '@/utils'
import {
  Clock,
  Globe,
  Lock,
  Monitor,
  Search,
  Shield,
  Smartphone,
  Target,
  Unlock,
  User,
} from 'lucide-react'
import { useMemo, useState } from 'react'
import { FlaggedUsers } from './flagged-users'
import { SecurityMetrics } from './security-metrics'
import { StrangeEventsAnalysis } from './strange-events-analysis'

import SecurityEvent = Schemas.SecurityEvent

interface PageOverviewProps {
  events: SecurityEvent[]
  isLoading: boolean
  isError: boolean
  realmName?: string
  isMocked: boolean
}

const formatTimestamp = (timestamp: string) => {
  const date = new Date(timestamp)
  if (Number.isNaN(date.getTime())) return 'Invalid date'
  return new Intl.DateTimeFormat(undefined, {
    dateStyle: 'medium',
    timeStyle: 'short',
  }).format(date)
}

const getDeviceIcon = (userAgent?: string | null) => {
  if (!userAgent) return Monitor
  const lower = userAgent.toLowerCase()
  if (lower.includes('iphone') || lower.includes('android') || lower.includes('mobile')) return Smartphone
  return Monitor
}

const getActorLabel = (event: SecurityEvent) => {
  return event.actor_id ?? event.target_id ?? 'Unknown actor'
}

const getStatusBadgeVariant = (status: SecurityEvent['status']) => {
  return status === 'failure' ? 'destructive' : 'default'
}

export default function PageOverview({
  events,
  isLoading,
  isError,
  realmName,
  isMocked,
}: PageOverviewProps) {
  const [query, setQuery] = useState('')
  const [statusFilter, setStatusFilter] = useState<'all' | 'success' | 'failure'>('all')

  const sortedEvents = useMemo(() => {
    return [...events].sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime())
  }, [events])

  const filteredEvents = useMemo(() => {
    const normalizedQuery = query.trim().toLowerCase()
    return sortedEvents.filter((event) => {
      if (statusFilter !== 'all' && event.status !== statusFilter) return false
      if (!normalizedQuery) return true
      const haystack = [
        event.event_type,
        event.actor_id,
        event.target_id,
        event.target_type,
        event.resource,
        event.ip_address,
        event.user_agent,
        event.status,
      ]
        .filter(Boolean)
        .join(' ')
        .toLowerCase()
      return haystack.includes(normalizedQuery)
    })
  }, [query, sortedEvents, statusFilter])

  return (
    <div className='min-h-screen bg-gradient-to-b from-background via-background to-muted/40'>
      <div className='container mx-auto flex flex-col gap-6 p-6 md:p-10 max-w-none'>
        <div className='flex flex-col gap-4 md:flex-row md:items-center md:justify-between'>
          <div className='flex items-center gap-4'>
            <div className='h-10 w-10 rounded-2xl bg-primary/10 flex items-center justify-center'>
              <Shield className='size-5 text-primary' />
            </div>
            <div>
              <Heading size={3} weight='medium'>SeaWatch</Heading>
              <div className='flex flex-wrap items-center gap-2 text-sm text-muted-foreground'>
                <span>Security events & audit signals</span>
                {realmName && (
                  <Badge variant='outline' className='border-primary/30 text-primary'>
                    Realm: {realmName}
                  </Badge>
                )}
                {isMocked ? (
                  <Badge variant='secondary'>Mock data</Badge>
                ) : (
                  <Badge variant='secondary'>Live feed</Badge>
                )}
              </div>
            </div>
          </div>

          <div className='flex flex-col gap-3 sm:flex-row sm:items-center'>
            <div className='relative'>
              <Search className='h-4 w-4 absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground' />
              <Input
                placeholder='Search events, actors, IPs...'
                className='pl-10 w-full sm:w-64'
                value={query}
                onChange={(event) => setQuery(event.target.value)}
              />
            </div>
          </div>
        </div>

        {isError && (
          <Alert variant='destructive'>
            <Shield />
            <AlertTitle>Security events unavailable</AlertTitle>
            <AlertDescription>
              We couldn&apos;t fetch the latest events. Showing cached or mocked data.
            </AlertDescription>
          </Alert>
        )}

        <SecurityMetrics events={events} isLoading={isLoading} />

        <div className='grid grid-cols-1 xl:grid-cols-3 gap-6'>
          <Card className='xl:col-span-2'>
            <CardHeader className='pb-4'>
              <div className='flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between'>
                <div>
                  <CardTitle className='flex items-center gap-2'>
                    <Shield className='h-4 w-4 text-primary' />
                    Event Stream
                  </CardTitle>
                  <p className='text-sm text-muted-foreground'>Latest activity in this realm.</p>
                </div>
                <Tabs value={statusFilter} onValueChange={(value) => setStatusFilter(value as typeof statusFilter)}>
                  <TabsList className='grid grid-cols-3'>
                    <TabsTrigger value='all'>All</TabsTrigger>
                    <TabsTrigger value='success'>Success</TabsTrigger>
                    <TabsTrigger value='failure'>Failure</TabsTrigger>
                  </TabsList>
                </Tabs>
              </div>
            </CardHeader>
            <CardContent>
              <div className='space-y-3'>
                {isLoading ? (
                  Array.from({ length: 4 }).map((_, index) => (
                    <div
                      key={`skeleton-event-${index}`}
                      className='flex flex-col gap-3 rounded-lg border border-border/60 bg-card/60 p-4'
                    >
                      <div className='flex items-center gap-3'>
                        <Skeleton className='h-9 w-9 rounded-full' />
                        <div className='flex-1 space-y-2'>
                          <div className='flex flex-wrap items-center gap-2'>
                            <Skeleton className='h-4 w-40' />
                            <Skeleton className='h-5 w-16 rounded-full' />
                            <Skeleton className='h-5 w-20 rounded-full' />
                          </div>
                          <div className='flex flex-wrap items-center gap-3'>
                            <Skeleton className='h-3 w-24' />
                            <Skeleton className='h-3 w-20' />
                            <Skeleton className='h-3 w-28' />
                          </div>
                        </div>
                        <Skeleton className='h-3 w-24' />
                      </div>
                      <Skeleton className='h-3 w-64' />
                    </div>
                  ))
                ) : (
                  <>
                    {filteredEvents.length === 0 && (
                      <div className='flex flex-col items-center justify-center py-16 text-muted-foreground'>
                        <Shield className='h-10 w-10 mb-4 opacity-40' />
                        <p>No events match your filters.</p>
                      </div>
                    )}
                    {filteredEvents.map((event) => {
                      const DeviceIcon = getDeviceIcon(event.user_agent)
                      const actorLabel = getActorLabel(event)
                      return (
                        <div
                          key={event.id}
                          className='flex flex-col gap-3 rounded-lg border border-border/70 bg-card/60 p-4 transition hover:border-primary/30 hover:shadow-sm'
                        >
                          <div className='flex flex-col gap-3 md:flex-row md:items-center md:justify-between'>
                            <div className='flex items-center gap-3'>
                              <div
                                className={`h-9 w-9 rounded-full flex items-center justify-center ${
                                  event.status === 'failure' ? 'bg-red-500/10' : 'bg-emerald-500/10'
                                }`}
                              >
                                {event.status === 'failure' ? (
                                  <Lock className='h-4 w-4 text-red-500' />
                                ) : (
                                  <Unlock className='h-4 w-4 text-emerald-500' />
                                )}
                              </div>
                              <div>
                                <div className='flex flex-wrap items-center gap-2'>
                                  <h4 className='font-medium'>{formatSnakeCaseToTitleCase(event.event_type)}</h4>
                                  <Badge variant={getStatusBadgeVariant(event.status)}>{event.status}</Badge>
                                  {event.resource && (
                                    <Badge variant='secondary'>{event.resource}</Badge>
                                  )}
                                </div>
                                <div className='mt-1 flex flex-wrap items-center gap-3 text-xs text-muted-foreground'>
                                  <span className='inline-flex items-center gap-1'>
                                    <User className='h-3 w-3' />
                                    {actorLabel}
                                  </span>
                                  {event.ip_address && (
                                    <span className='inline-flex items-center gap-1'>
                                      <Globe className='h-3 w-3' />
                                      {event.ip_address}
                                    </span>
                                  )}
                                  {event.target_id && (
                                    <span className='inline-flex items-center gap-1'>
                                      <Target className='h-3 w-3' />
                                      {event.target_id}
                                    </span>
                                  )}
                                  <span className='inline-flex items-center gap-1'>
                                    <DeviceIcon className='h-3 w-3' />
                                    {event.user_agent ?? 'Unknown device'}
                                  </span>
                                </div>
                              </div>
                            </div>
                            <div className='text-sm text-muted-foreground inline-flex items-center gap-1'>
                              <Clock className='h-3 w-3' />
                              {formatTimestamp(event.timestamp)}
                            </div>
                          </div>
                          {Boolean(event.details) && (
                            <div className='text-xs text-muted-foreground border-t border-border/60 pt-3'>
                              <span className='font-medium text-foreground'>Details:</span>{' '}
                              {typeof event.details === 'string' ? event.details : 'Additional context available'}
                            </div>
                          )}
                        </div>
                      )
                    })}
                  </>
                )}
              </div>
            </CardContent>
          </Card>

          <div className='flex flex-col gap-6'>
            {isLoading ? (
              <>
                <Card>
                  <CardHeader className='pb-4'>
                    <Skeleton className='h-5 w-32' />
                  </CardHeader>
                  <CardContent className='space-y-4'>
                    {Array.from({ length: 3 }).map((_, index) => (
                      <div key={`skeleton-breakdown-${index}`} className='space-y-2'>
                        <div className='flex items-center justify-between'>
                          <Skeleton className='h-3 w-24' />
                          <Skeleton className='h-3 w-8' />
                        </div>
                        <Skeleton className='h-2 w-full rounded-full' />
                      </div>
                    ))}
                  </CardContent>
                </Card>
                <Card>
                  <CardHeader className='pb-4'>
                    <Skeleton className='h-5 w-28' />
                  </CardHeader>
                  <CardContent className='space-y-3'>
                    {Array.from({ length: 3 }).map((_, index) => (
                      <div key={`skeleton-actor-${index}`} className='flex items-center gap-3'>
                        <Skeleton className='h-8 w-8 rounded-full' />
                        <div className='flex-1 space-y-2'>
                          <Skeleton className='h-3 w-32' />
                          <Skeleton className='h-2 w-24' />
                        </div>
                        <Skeleton className='h-6 w-10 rounded-full' />
                      </div>
                    ))}
                  </CardContent>
                </Card>
              </>
            ) : (
              <>
                <StrangeEventsAnalysis events={events} />
                <FlaggedUsers events={events} />
              </>
            )}
          </div>
        </div>
      </div>
    </div>
  )
}
