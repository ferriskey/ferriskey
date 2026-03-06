import { Schemas } from '@/api/api.client'
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Heading } from '@/components/ui/heading'
import { Skeleton } from '@/components/ui/skeleton'
import { COMPASS_OVERVIEW_URL } from '@/routes/sub-router/compass.router'
import { formatSnakeCaseToTitleCase } from '@/utils'
import { ArrowLeft, Clock, Compass, Globe, Monitor, User } from 'lucide-react'
import { useNavigate, useParams } from 'react-router'
import { RouterParams } from '@/routes/router'
import { FlowTimeline } from './flow-timeline'

import CompassFlow = Schemas.CompassFlow
import FlowStatus = Schemas.FlowStatus

interface PageFlowDetailProps {
  flow: CompassFlow | null
  isLoading: boolean
  isError: boolean
}

const getStatusVariant = (status: FlowStatus): 'default' | 'destructive' | 'secondary' | 'outline' => {
  switch (status) {
    case 'success':
      return 'default'
    case 'failure':
      return 'destructive'
    case 'pending':
      return 'secondary'
    case 'expired':
      return 'outline'
  }
}

const formatTimestamp = (timestamp: string) => {
  const date = new Date(timestamp)
  if (Number.isNaN(date.getTime())) return 'Invalid date'
  return new Intl.DateTimeFormat(undefined, {
    dateStyle: 'medium',
    timeStyle: 'medium',
  }).format(date)
}

export default function PageFlowDetail({ flow, isLoading, isError }: PageFlowDetailProps) {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()

  if (isLoading) {
    return (
      <div className='min-h-screen bg-gradient-to-b from-background via-background to-muted/40'>
        <div className='container mx-auto flex flex-col gap-6 p-6 md:p-10 max-w-none'>
          <Skeleton className='h-8 w-48' />
          <div className='grid grid-cols-1 sm:grid-cols-4 gap-4'>
            {Array.from({ length: 4 }).map((_, i) => (
              <Skeleton key={`skeleton-detail-${i}`} className='h-24' />
            ))}
          </div>
          <Skeleton className='h-48' />
        </div>
      </div>
    )
  }

  if (isError || !flow) {
    return (
      <div className='min-h-screen bg-gradient-to-b from-background via-background to-muted/40'>
        <div className='container mx-auto flex flex-col gap-6 p-6 md:p-10 max-w-none'>
          <Alert variant='destructive'>
            <Compass className='h-4 w-4' />
            <AlertTitle>Flow not found</AlertTitle>
            <AlertDescription>
              The flow you&apos;re looking for doesn&apos;t exist or couldn&apos;t be loaded.
            </AlertDescription>
          </Alert>
        </div>
      </div>
    )
  }

  return (
    <div className='min-h-screen bg-gradient-to-b from-background via-background to-muted/40'>
      <div className='container mx-auto flex flex-col gap-6 p-6 md:p-10 max-w-none'>
        {/* Header */}
        <div className='flex items-center gap-4'>
          <button
            onClick={() => navigate(COMPASS_OVERVIEW_URL(realm_name))}
            className='h-9 w-9 rounded-lg border border-border flex items-center justify-center hover:bg-muted transition-colors'
          >
            <ArrowLeft className='h-4 w-4' />
          </button>
          <div>
            <div className='flex items-center gap-3'>
              <Heading size={3} weight='medium'>
                {formatSnakeCaseToTitleCase(flow.grant_type)}
              </Heading>
              <Badge variant={getStatusVariant(flow.status)}>{flow.status}</Badge>
              {flow.duration_ms != null && (
                <Badge variant='outline'>{flow.duration_ms}ms</Badge>
              )}
            </div>
            <p className='text-sm text-muted-foreground font-mono'>{flow.id}</p>
          </div>
        </div>

        {/* Metadata cards */}
        <div className='grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4'>
          <Card>
            <CardHeader className='pb-2'>
              <CardTitle className='text-sm font-medium text-muted-foreground flex items-center gap-1'>
                <Clock className='h-3 w-3' /> Started
              </CardTitle>
            </CardHeader>
            <CardContent>
              <p className='text-sm font-medium'>{formatTimestamp(flow.started_at)}</p>
            </CardContent>
          </Card>
          {flow.completed_at && (
            <Card>
              <CardHeader className='pb-2'>
                <CardTitle className='text-sm font-medium text-muted-foreground flex items-center gap-1'>
                  <Clock className='h-3 w-3' /> Completed
                </CardTitle>
              </CardHeader>
              <CardContent>
                <p className='text-sm font-medium'>{formatTimestamp(flow.completed_at)}</p>
              </CardContent>
            </Card>
          )}
          {flow.client_id && (
            <Card>
              <CardHeader className='pb-2'>
                <CardTitle className='text-sm font-medium text-muted-foreground flex items-center gap-1'>
                  <Monitor className='h-3 w-3' /> Client
                </CardTitle>
              </CardHeader>
              <CardContent>
                <p className='text-sm font-medium font-mono'>{flow.client_id}</p>
              </CardContent>
            </Card>
          )}
          {flow.user_id && (
            <Card>
              <CardHeader className='pb-2'>
                <CardTitle className='text-sm font-medium text-muted-foreground flex items-center gap-1'>
                  <User className='h-3 w-3' /> User
                </CardTitle>
              </CardHeader>
              <CardContent>
                <p className='text-sm font-medium font-mono'>{flow.user_id}</p>
              </CardContent>
            </Card>
          )}
          {flow.ip_address && (
            <Card>
              <CardHeader className='pb-2'>
                <CardTitle className='text-sm font-medium text-muted-foreground flex items-center gap-1'>
                  <Globe className='h-3 w-3' /> IP Address
                </CardTitle>
              </CardHeader>
              <CardContent>
                <p className='text-sm font-medium font-mono'>{flow.ip_address}</p>
              </CardContent>
            </Card>
          )}
        </div>

        {/* Flow graph */}
        <div>
          <h2 className='text-base font-semibold flex items-center gap-2 mb-4'>
            <Compass className='h-4 w-4 text-primary' />
            Flow Steps
          </h2>
          <div className='rounded-lg border border-border bg-card/40 overflow-hidden'>
            <FlowTimeline steps={flow.steps} />
          </div>
        </div>
      </div>
    </div>
  )
}
