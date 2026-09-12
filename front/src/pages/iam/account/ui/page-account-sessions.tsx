import { LogOut, Monitor, UserRound } from 'lucide-react'
import { useLocation, useParams } from 'react-router'
import { ConfirmDeleteAlert } from '@/components/confirm-delete-alert'
import { useConfirmDeleteAlert } from '@/hooks/use-confirm-delete-alert.ts'
import { Button, DetailHeader, IconTile, PageShell, PageTabs, Pill, Section } from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import { ACCOUNT_URL, RouterParams } from '@/routes/router'
import { formatDate, formatDateTime } from '@/utils/format-date'
import { Schemas } from '@/api/api.client'

import User = Schemas.User
import UserSessionDto = Schemas.UserSessionDto

export interface PageAccountSessionsProps {
  profile?: User
  isLoading: boolean
  sessions: UserSessionDto[]
  currentSessionId: string | null
  onRevoke: (sessionId: string) => void
}

export default function PageAccountSessions({
  profile,
  isLoading,
  sessions,
  currentSessionId,
  onRevoke,
}: PageAccountSessionsProps) {
  const { realm_name } = useParams<RouterParams>()
  const { pathname } = useLocation()
  const { confirm, ask, close } = useConfirmDeleteAlert()

  const base = ACCOUNT_URL(realm_name)
  const tabs = [
    { key: 'overview', label: 'Personal info', href: base },
    { key: 'sessions', label: 'Sessions', href: `${base}/sessions` },
  ]
  const tab = pathname.endsWith('/sessions') ? 'sessions' : 'overview'

  const askRevoke = (session: UserSessionDto) =>
    ask({
      title: 'Revoke session?',
      description: `This will immediately sign out the session on ${session.user_agent ?? 'this device'}.`,
      onConfirm: () => {
        onRevoke(session.id)
        close()
      },
    })

  if (isLoading) {
    return (
      <PageShell>
        <div className='flex items-center gap-3'>
          <div className='size-15 animate-pulse rounded-md bg-neutral-100 dark:bg-fk-raised' />
          <div className='space-y-2'>
            <div className='h-5 w-48 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
            <div className='h-4 w-32 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
          </div>
        </div>
      </PageShell>
    )
  }

  if (!profile) {
    return (
      <PageShell>
        <div className={cn(tokens.surface.panel, 'grid place-items-center px-6 py-16')}>
          <p className='text-sm font-medium text-neutral-700 dark:text-neutral-300'>Profile unavailable</p>
          <p className='mt-1 max-w-sm text-center text-sm text-neutral-500 dark:text-neutral-400'>
            Your account could not be loaded for this realm. Sign in again, then retry.
          </p>
        </div>
      </PageShell>
    )
  }

  return (
    <PageShell>
      <DetailHeader
        icon={
          <IconTile tone='primary' className='size-15'>
            <UserRound className='size-6' strokeWidth={1.75} />
          </IconTile>
        }
        title={profile.username}
        pills={
          <Pill tone={profile.email_verified ? 'info' : 'amber'} mono>
            {profile.email_verified ? 'email verified' : 'email unverified'}
          </Pill>
        }
        meta={
          <dl className='shrink-0 text-right text-xs text-neutral-500 dark:text-neutral-400'>
            <dt className='sr-only'>Member since</dt>
            <dd className='tnum'>
              Member since{' '}
              {formatDate(profile.created_at)}
            </dd>
            <dt className='sr-only'>Identifier</dt>
            <dd className='font-mono-ui text-[11px] text-neutral-400 dark:text-neutral-500'>{profile.id}</dd>
          </dl>
        }
      />

      <PageTabs tabs={tabs} value={tab} className='mt-5' />

      <div className={cn('mt-5', tokens.page.blockGap)}>
        <Section title={`Active sessions (${sessions.length})`} contained={sessions.length > 0}>
          {sessions.length > 0 ? (
            <ul className={cn(tokens.surface.divider, tokens.table.text)}>
              {sessions.map((session) => {
                const isCurrent = session.id === currentSessionId
                return (
                  <li key={session.id} className='flex items-center gap-3 py-2.5'>
                    <IconTile tone='info' className='size-7'>
                      <Monitor className='size-3.5' strokeWidth={1.75} />
                    </IconTile>

                    <div className='min-w-0 flex-1'>
                      <p className='flex items-center gap-1.5 truncate font-medium text-neutral-900 dark:text-neutral-100'>
                        {session.user_agent ?? 'Unknown device'}
                        {isCurrent && (
                          <Pill tone='success' mono>
                            current
                          </Pill>
                        )}
                      </p>
                      <p className='mt-0.5 truncate text-xs text-neutral-500 dark:text-neutral-400'>
                        {session.ip_address ?? 'Unknown IP'} · last seen{' '}
                        {session.last_seen_at ? formatDateTime(session.last_seen_at) : 'never'} · signed in{' '}
                        {formatDateTime(session.created_at)}
                      </p>
                    </div>

                    {!isCurrent && (
                      <Button
                        variant='ghost'
                        size='sm'
                        aria-label={`Revoke the session on ${session.user_agent ?? 'this device'}`}
                        onClick={() => askRevoke(session)}
                        className='shrink-0 text-neutral-400 dark:text-neutral-500 hover:text-fk-danger'
                      >
                        <LogOut className='size-3.5' />
                        Revoke
                      </Button>
                    )}
                  </li>
                )
              })}
            </ul>
          ) : (
            <p className='rounded-lg border border-dashed border-fk-line px-4 py-3 text-xs text-neutral-500 dark:text-neutral-400'>
              No active sessions.
            </p>
          )}
        </Section>
      </div>

      <ConfirmDeleteAlert
        title={confirm.title}
        description={confirm.description}
        open={confirm.open}
        onConfirm={confirm.onConfirm}
        onCancel={close}
      />
    </PageShell>
  )
}
