import { LogOut, Monitor } from 'lucide-react'
import { OverviewList } from '@/components/ui/overview-list'
import { EntityAvatar } from '@/components/ui/entity-avatar'
import { ConfirmDeleteAlert } from '@/components/confirm-delete-alert'
import { Button } from '@/components/ui/button'
import { useConfirmDeleteAlert } from '@/hooks/use-confirm-delete-alert.ts'
import { Schemas } from '@/api/api.client'

import UserSessionDto = Schemas.UserSessionDto

export interface PageAccountSessionsProps {
  sessions: UserSessionDto[]
  currentSessionId: string | null
  onRevoke: (sessionId: string) => void
}

function formatDate(value: string) {
  return new Date(value).toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'short',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  })
}

export default function PageAccountSessions({
  sessions,
  currentSessionId,
  onRevoke,
}: PageAccountSessionsProps) {
  const { confirm, ask, close } = useConfirmDeleteAlert()

  const onRowRevoke = (session: UserSessionDto) => {
    ask({
      title: 'Revoke session?',
      description: `This will immediately sign out the session on ${session.user_agent ?? 'this device'}.`,
      onConfirm: () => {
        onRevoke(session.id)
        close()
      },
    })
  }

  return (
    <div className='flex flex-col gap-6'>
      <OverviewList
        data={sessions}
        title={(n) => `Active sessions (${n})`}
        emptyLabel='No active sessions.'
        renderRow={(session) => {
          const isCurrent = session.id === currentSessionId
          return (
            <div className='flex items-center justify-between px-8 py-4 border-b last:border-b-0 hover:bg-muted/40 transition-colors'>
              <div className='flex items-center gap-4'>
                <EntityAvatar label={session.user_agent ?? 'device'} color='#6366F1' />
                <div>
                  <div className='flex items-center gap-2.5'>
                    <span className='text-base font-medium flex items-center gap-1.5'>
                      <Monitor className='h-4 w-4 text-muted-foreground' />
                      {session.user_agent ?? 'Unknown device'}
                    </span>
                    {isCurrent && (
                      <span className='inline-flex items-center px-2.5 py-0.5 rounded-md border text-xs font-mono border-green-300 text-green-600 bg-green-50 dark:bg-green-500/10 dark:border-green-400/40'>
                        current
                      </span>
                    )}
                  </div>
                  <div className='text-sm text-muted-foreground mt-0.5'>
                    {session.ip_address ?? 'Unknown IP'} · last seen{' '}
                    {session.last_seen_at ? formatDate(session.last_seen_at) : 'never'} · signed in{' '}
                    {formatDate(session.created_at)}
                  </div>
                </div>
              </div>
              <div className='flex items-center gap-2'>
                {!isCurrent && (
                  <Button
                    variant='ghost'
                    size='sm'
                    className='text-destructive hover:text-destructive hover:bg-destructive/10'
                    onClick={() => onRowRevoke(session)}
                  >
                    <LogOut className='h-4 w-4 mr-1.5' />
                    Revoke
                  </Button>
                )}
              </div>
            </div>
          )
        }}
      />

      <ConfirmDeleteAlert
        title={confirm.title}
        description={confirm.description}
        open={confirm.open}
        onConfirm={confirm.onConfirm}
        onCancel={close}
      />
    </div>
  )
}
