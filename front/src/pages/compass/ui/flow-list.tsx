import { Schemas } from '@/api/api.client'
import { Skeleton } from '@/components/ui/skeleton'
import { formatSnakeCaseToTitleCase } from '@/utils'
import { Compass } from 'lucide-react'
import { useNavigate, useParams } from 'react-router'
import { RouterParams } from '@/routes/router'
import { COMPASS_FLOW_DETAIL_URL } from '@/routes/sub-router/compass.router'

import CompassFlow = Schemas.CompassFlow
import FlowStatus = Schemas.FlowStatus

interface FlowListProps {
  flows: CompassFlow[]
  isLoading: boolean
}

const formatTimestamp = (timestamp: string) => {
  const date = new Date(timestamp)
  if (Number.isNaN(date.getTime())) return 'Invalid date'
  return new Intl.DateTimeFormat(undefined, {
    dateStyle: 'medium',
    timeStyle: 'short',
  }).format(date)
}

const grantTypeInitial = (grantType: string) => {
  const map: Record<string, string> = {
    authorization_code: 'A',
    password: 'P',
    client_credentials: 'C',
    refresh_token: 'R',
  }
  return map[grantType] ?? grantType[0]?.toUpperCase() ?? '?'
}

const grantTypeColor = (grantType: string) => {
  const map: Record<string, string> = {
    authorization_code: '#3B82F6',
    password: '#F97316',
    client_credentials: '#8B5CF6',
    refresh_token: '#06B6D4',
  }
  return map[grantType] ?? '#6B7280'
}

function FlowAvatar({ grantType }: { grantType: string }) {
  return (
    <div
      className='h-10 w-10 rounded-md flex items-center justify-center shrink-0'
      style={{ backgroundColor: grantTypeColor(grantType) }}
    >
      <span className='text-base font-bold text-white'>{grantTypeInitial(grantType)}</span>
    </div>
  )
}

function FlowStatusBadge({ status }: { status: FlowStatus }) {
  switch (status) {
    case 'success':
      return (
        <span className='inline-flex items-center px-3 py-1 rounded-md text-xs font-semibold border border-emerald-400/50 text-emerald-600 bg-emerald-50 dark:bg-emerald-500/10'>
          SUCCESS
        </span>
      )
    case 'failure':
      return (
        <span className='inline-flex items-center px-3 py-1 rounded-md text-xs font-semibold border border-red-400/50 text-red-500 bg-red-50 dark:bg-red-500/10'>
          FAILURE
        </span>
      )
    case 'pending':
      return (
        <span className='inline-flex items-center px-3 py-1 rounded-md text-xs font-semibold border border-amber-400/50 text-amber-600 bg-amber-50 dark:bg-amber-500/10'>
          PENDING
        </span>
      )
    case 'expired':
      return (
        <span className='inline-flex items-center px-3 py-1 rounded-md text-xs font-semibold border border-gray-400/50 text-gray-500 bg-gray-50 dark:bg-gray-500/10'>
          EXPIRED
        </span>
      )
  }
}

function DurationBadge({ durationMs }: { durationMs?: number | null }) {
  if (durationMs == null) return null
  return (
    <span className='inline-flex items-center px-2.5 py-0.5 rounded-md border border-border text-xs font-mono text-muted-foreground'>
      {durationMs}ms
    </span>
  )
}

export function FlowList({ flows, isLoading }: FlowListProps) {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()

  if (isLoading) {
    return (
      <div className='-mx-8 border-t border-b overflow-hidden'>
        {Array.from({ length: 5 }).map((_, i) => (
          <div key={`skeleton-flow-${i}`} className='flex items-center justify-between px-8 py-4 border-b last:border-b-0'>
            <div className='flex items-center gap-3'>
              <Skeleton className='h-10 w-10 rounded-md' />
              <div className='space-y-2'>
                <Skeleton className='h-4 w-40' />
                <Skeleton className='h-3 w-32' />
              </div>
            </div>
            <Skeleton className='h-6 w-16 rounded-md' />
          </div>
        ))}
      </div>
    )
  }

  if (flows.length === 0) {
    return (
      <div className='-mx-8 border-t border-b overflow-hidden'>
        <div className='flex flex-col items-center justify-center h-24 text-sm text-muted-foreground'>
          <Compass className='h-5 w-5 mb-2 opacity-40' />
          No flows match your filters.
        </div>
      </div>
    )
  }

  return (
    <div className='-mx-8 border-t border-b overflow-hidden'>
      {flows.map((flow) => (
        <div
          key={flow.id}
          onClick={() => navigate(COMPASS_FLOW_DETAIL_URL(realm_name, flow.id))}
          className='flex items-center justify-between px-8 py-4 border-b last:border-b-0 hover:bg-muted/40 cursor-pointer transition-colors'
        >
          <div className='flex items-center gap-4'>
            <FlowAvatar grantType={flow.grant_type} />
            <div>
              <div className='flex items-center gap-2.5'>
                <span className='text-base font-medium'>{formatSnakeCaseToTitleCase(flow.grant_type)}</span>
                <DurationBadge durationMs={flow.duration_ms} />
              </div>
              <div className='text-sm text-muted-foreground mt-0.5'>
                {flow.client_id && <span>client: {flow.client_id}</span>}
                {flow.client_id && flow.started_at && <span className='mx-1.5'>·</span>}
                <span>{formatTimestamp(flow.started_at)}</span>
              </div>
            </div>
          </div>

          <FlowStatusBadge status={flow.status} />
        </div>
      ))}
    </div>
  )
}
