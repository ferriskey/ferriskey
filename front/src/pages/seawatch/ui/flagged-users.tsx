import { Schemas } from '@/api/api.client'
import { Avatar, AvatarFallback } from '@/components/ui/avatar'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { AlertTriangle, User } from 'lucide-react'
import { useMemo } from 'react'

import SecurityEvent = Schemas.SecurityEvent

export interface FlaggedUsersProps {
  events: SecurityEvent[]
}

const getDisplayName = (identifier: string) => {
  if (!identifier) return 'Unknown actor'
  if (identifier.includes('@')) return identifier
  return identifier.replace(/[_-]/g, ' ')
}

const getAvatarFallback = (identifier: string) => {
  const cleaned = identifier.replace(/[^a-zA-Z0-9 ]/g, ' ')
  const parts = cleaned.trim().split(/\s+/).filter(Boolean)
  if (!parts.length) return 'UN'
  if (parts.length === 1) return parts[0].slice(0, 2).toUpperCase()
  return `${parts[0][0]}${parts[1][0]}`.toUpperCase()
}

const formatTimestamp = (timestamp: string) => {
  const date = new Date(timestamp)
  if (Number.isNaN(date.getTime())) return 'unknown time'
  return new Intl.DateTimeFormat(undefined, {
    dateStyle: 'short',
    timeStyle: 'short',
  }).format(date)
}

export function FlaggedUsers({ events }: FlaggedUsersProps) {
  const flaggedUsers = useMemo(() => {
    const failureEvents = events.filter((event) => event.status === 'failure')
    const grouped = new Map<string, { count: number; lastSeen: string; ip?: string | null }>()

    failureEvents.forEach((event) => {
      const key = event.actor_id ?? 'Unknown actor'
      const existing = grouped.get(key)
      if (!existing) {
        grouped.set(key, {
          count: 1,
          lastSeen: event.timestamp,
          ip: event.ip_address,
        })
      } else {
        existing.count += 1
        if (new Date(event.timestamp).getTime() > new Date(existing.lastSeen).getTime()) {
          existing.lastSeen = event.timestamp
          existing.ip = event.ip_address ?? existing.ip
        }
      }
    })

    return Array.from(grouped.entries())
      .map(([identifier, data]) => ({
        identifier,
        count: data.count,
        lastSeen: data.lastSeen,
        ip: data.ip,
      }))
      .sort((a, b) => b.count - a.count)
      .slice(0, 4)
  }, [events])

  return (
    <div>
      <Card>
        <CardHeader className='pb-4'>
          <CardTitle className='flex items-center gap-2'>
            <User className='h-4 w-4 text-amber-500' />
            Risky Actors
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className='space-y-3'>
            {flaggedUsers.length === 0 && (
              <div className='rounded-lg border border-dashed border-muted p-6 text-center text-sm text-muted-foreground'>
                No risky actors detected.
              </div>
            )}
            {flaggedUsers.map((user) => (
              <div
                key={user.identifier}
                className='flex items-center space-x-3 border rounded-md p-3 hover:shadow-sm transition-all bg-card/70'
              >
                <Avatar className='h-8 w-8'>
                  <AvatarFallback className='bg-primary/10 text-primary font-medium text-xs'>
                    {getAvatarFallback(user.identifier)}
                  </AvatarFallback>
                </Avatar>
                <div className='flex-1 min-w-0'>
                  <p className='font-medium truncate'>
                    {getDisplayName(user.identifier)}
                  </p>
                  <p className='text-xs text-muted-foreground truncate'>
                    {user.ip ? `Last seen from ${user.ip}` : 'Last seen location unavailable'}
                  </p>
                  <p className='text-[11px] text-muted-foreground'>
                    Last event {formatTimestamp(user.lastSeen)}
                  </p>
                </div>
                <div className='text-right flex items-center gap-2'>
                  <Badge variant={user.count > 3 ? 'destructive' : 'secondary'}>
                    {user.count}x
                  </Badge>
                  <AlertTriangle className='h-4 w-4 text-amber-500' />
                </div>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
