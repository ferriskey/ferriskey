import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import {
  Settings,
  Users,
  Clock,
  Server,
  ChevronRight,
  CircleDot,
  LucideIcon,
} from 'lucide-react'

interface ProviderCardProps {
  name: string
  type: string
  status: 'active' | 'syncing' | 'inactive'
  users: number
  lastSync: string
  connection: string
  priority: string
  icon: LucideIcon
  onClick?: () => void
  onSettings?: () => void
}

export default function ProviderCard({
  name,
  type,
  status,
  users,
  lastSync,
  connection,
  priority,
  icon: Icon,
  onClick,
  onSettings,
}: ProviderCardProps) {
  return (
    <div
      onClick={onClick}
      className='group flex items-center gap-4 p-4 border rounded-lg hover:bg-accent/50 transition-all cursor-pointer'
    >
      {/* Icon */}
      <div className='flex h-10 w-10 items-center justify-center rounded-lg bg-primary/10 shrink-0'>
        <Icon className='h-5 w-5 text-primary' />
      </div>

      {/* Main Info */}
      <div className='flex-1 min-w-0'>
        <div className='flex items-center gap-2 mb-1'>
          <h4 className='font-medium text-sm'>{name}</h4>
          <Badge variant='outline' className='text-xs'>
            {type}
          </Badge>
          <Badge variant='outline' className='text-xs'>
            {priority}
          </Badge>

          <div className='flex items-center gap-2'>
            <CircleDot
              className={`h-3 w-3 ${status === 'active'
                ? 'text-green-500 fill-green-500'
                : status === 'syncing'
                  ? 'text-blue-500 fill-blue-500 animate-pulse'
                  : 'text-gray-400 fill-gray-400'
                }`}
            />
            <span className='text-xs text-muted-foreground capitalize'>{status}</span>
          </div>
        </div>
        <div className='flex items-center gap-3 text-xs text-muted-foreground'>
          <span className='flex items-center gap-1'>
            <Server className='h-3 w-3' />
            {connection}
          </span>
          <span className='flex items-center gap-1'>
            <Users className='h-3 w-3' />
            {users.toLocaleString()} users
          </span>
        </div>
      </div>

      {/* Status */}
      <div className='flex items-center gap-3 shrink-0'>
        <div className='flex items-center gap-2'>
          <CircleDot
            className={`h-3 w-3 ${status === 'active'
              ? 'text-green-500 fill-green-500'
              : status === 'syncing'
                ? 'text-blue-500 fill-blue-500 animate-pulse'
                : 'text-gray-400 fill-gray-400'
              }`}
          />
          <span className='text-xs text-muted-foreground capitalize'>{status}</span>
        </div>
        <div className='flex items-center gap-1 text-xs text-muted-foreground'>
          <Clock className='h-3 w-3' />
          {lastSync}
        </div>
      </div>

      {/* Actions */}
      <div className='flex items-center gap-1 shrink-0'>
        <Button
          variant='ghost'
          size='sm'
          onClick={(e) => {
            e.stopPropagation()
            onSettings?.()
          }}
          className='opacity-0 group-hover:opacity-100 transition-opacity'
        >
          <Settings className='h-4 w-4' />
        </Button>
        <ChevronRight className='h-4 w-4 text-muted-foreground' />
      </div>
    </div>
  )
}
