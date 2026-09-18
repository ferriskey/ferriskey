import { LogOut, Monitor, UserRound } from 'lucide-react'
import { useLocation, useParams } from 'react-router'
import { useTranslation } from 'react-i18next'
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
  const { t } = useTranslation('account')
  const { realm_name } = useParams<RouterParams>()
  const { pathname } = useLocation()
  const { confirm, ask, close } = useConfirmDeleteAlert()

  const base = ACCOUNT_URL(realm_name)
  const tabs = [
    { key: 'overview', label: t('tabs.overview'), href: base },
    { key: 'sessions', label: t('tabs.sessions'), href: `${base}/sessions` },
  ]
  const tab = pathname.endsWith('/sessions') ? 'sessions' : 'overview'

  const deviceName = (session: UserSessionDto) => session.user_agent ?? t('sessions.this_device')

  const askRevoke = (session: UserSessionDto) =>
    ask({
      title: t('sessions.confirm.title'),
      description: t('sessions.confirm.description', { device: deviceName(session) }),
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
          <p className='text-sm font-medium text-neutral-700 dark:text-neutral-300'>
            {t('unavailable.title')}
          </p>
          <p className='mt-1 max-w-sm text-center text-sm text-neutral-500 dark:text-neutral-400'>
            {t('unavailable.description')}
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
            {profile.email_verified ? t('header.email_verified') : t('header.email_unverified')}
          </Pill>
        }
        meta={
          <dl className='shrink-0 text-right text-xs text-neutral-500 dark:text-neutral-400'>
            <dt className='sr-only'>{t('header.member_since_label')}</dt>
            <dd className='tnum'>
              {t('header.member_since', { date: formatDate(profile.created_at) })}
            </dd>
            <dt className='sr-only'>{t('header.identifier_label')}</dt>
            <dd className='font-mono-ui text-[11px] text-neutral-400 dark:text-neutral-500'>{profile.id}</dd>
          </dl>
        }
      />

      <PageTabs tabs={tabs} value={tab} className='mt-5' />

      <div className={cn('mt-5', tokens.page.blockGap)}>
        <Section
          title={t('sessions.title', { total: sessions.length })}
          contained={sessions.length > 0}
        >
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
                        {session.user_agent ?? t('sessions.unknown_device')}
                        {isCurrent && (
                          <Pill tone='success' mono>
                            {t('sessions.current')}
                          </Pill>
                        )}
                      </p>
                      <p className='mt-0.5 truncate text-xs text-neutral-500 dark:text-neutral-400'>
                        {t('sessions.meta', {
                          ip: session.ip_address ?? t('sessions.unknown_ip'),
                          lastSeen: session.last_seen_at
                            ? formatDateTime(session.last_seen_at)
                            : t('sessions.never'),
                          signedIn: formatDateTime(session.created_at),
                        })}
                      </p>
                    </div>

                    {!isCurrent && (
                      <Button
                        variant='ghost'
                        size='sm'
                        aria-label={t('sessions.revoke_label', { device: deviceName(session) })}
                        onClick={() => askRevoke(session)}
                        className='shrink-0 text-neutral-400 dark:text-neutral-500 hover:text-fk-danger'
                      >
                        <LogOut className='size-3.5' />
                        {t('sessions.revoke')}
                      </Button>
                    )}
                  </li>
                )
              })}
            </ul>
          ) : (
            <p className='rounded-lg border border-dashed border-fk-line px-4 py-3 text-xs text-neutral-500 dark:text-neutral-400'>
              {t('sessions.empty')}
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
