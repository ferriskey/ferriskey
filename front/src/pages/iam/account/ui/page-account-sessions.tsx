import { LogOut, Monitor, UserRound } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { useAccountTabs } from './use-account-tabs'
import { ConfirmDeleteAlert } from '@/components/confirm-delete-alert'
import { useConfirmDeleteAlert } from '@/hooks/use-confirm-delete-alert.ts'
import {
  Button,
  DataView,
  DetailHeader,
  FilterBar,
  IconTile,
  PageShell,
  PageTabs,
  PaginationBar,
  Pill,
  Section,
  type Column,
  type FilterField,
  type PagedListing,
  type PaginationMetadata,
  type ViewMode,
} from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import { formatDate, formatDateTime } from '@/utils/format-date'
import { Schemas } from '@/api/api.client'

import User = Schemas.User
import UserSessionDto = Schemas.UserSessionDto

const SESSION_VIEW: ViewMode = 'list'

export interface PageAccountSessionsProps {
  profile?: User
  isLoading: boolean
  isSessionsLoading: boolean
  sessions: UserSessionDto[]
  pagination?: PaginationMetadata
  listing: PagedListing
  currentSessionId: string | null
  onRevoke: (sessionId: string) => void
}

export default function PageAccountSessions({
  profile,
  isLoading,
  isSessionsLoading,
  sessions,
  pagination,
  listing,
  currentSessionId,
  onRevoke,
}: PageAccountSessionsProps) {
  const { t } = useTranslation('account')
  const { confirm, ask, close } = useConfirmDeleteAlert()

  const { tabs, tab } = useAccountTabs()

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

  const narrowed = Object.values(listing.state.filters).some(Boolean)
  const total = pagination?.total ?? sessions.length

  const filterFields: FilterField[] = [
    { kind: 'text', key: 'user_agent', label: t('sessions.filter_fields.user_agent') },
    { kind: 'text', key: 'ip_address', label: t('sessions.filter_fields.ip_address') },
    { kind: 'boolean', key: 'persistent', label: t('sessions.filter_fields.persistent') },
  ]

  const columns: Column<UserSessionDto>[] = [
    {
      key: 'device',
      header: t('sessions.columns.device'),
      render: (session) => (
        <div className='flex min-w-0 items-center gap-3'>
          <IconTile tone='info' className='size-7'>
            <Monitor className='size-3.5' strokeWidth={1.75} />
          </IconTile>
          <p className='flex min-w-0 items-center gap-1.5 truncate font-medium text-neutral-900 dark:text-neutral-100'>
            <span className='truncate'>{session.user_agent ?? t('sessions.unknown_device')}</span>
            {session.id === currentSessionId && (
              <Pill tone='success' mono>
                {t('sessions.current')}
              </Pill>
            )}
          </p>
        </div>
      ),
    },
    {
      key: 'ip_address',
      header: t('sessions.columns.ip_address'),
      render: (session) => (
        <span className='font-mono-ui text-[12px] text-neutral-600 dark:text-neutral-400'>
          {session.ip_address ?? t('sessions.unknown_ip')}
        </span>
      ),
    },
    {
      key: 'persistent',
      header: t('sessions.columns.persistent'),
      render: (session) => (
        <Pill tone={session.persistent ? 'info' : 'neutral'} mono>
          {session.persistent ? t('sessions.persistent_state.remembered') : t('sessions.persistent_state.browser')}
        </Pill>
      ),
    },
    {
      key: 'last_seen_at',
      header: t('sessions.columns.last_seen_at'),
      align: 'right',
      render: (session) => (
        <span className='tnum whitespace-nowrap'>
          {session.last_seen_at ? formatDateTime(session.last_seen_at) : t('sessions.never')}
        </span>
      ),
      sortKey: 'last_seen_at',
    },
    {
      key: 'expires_at',
      header: t('sessions.columns.expires_at'),
      align: 'right',
      render: (session) => (
        <span className='tnum whitespace-nowrap'>{formatDateTime(session.expires_at)}</span>
      ),
      sortKey: 'expires_at',
    },
    {
      key: 'created_at',
      header: t('sessions.columns.created_at'),
      align: 'right',
      render: (session) => (
        <span className='tnum whitespace-nowrap'>{formatDateTime(session.created_at)}</span>
      ),
      sortKey: 'created_at',
    },
    {
      key: 'actions',
      header: '',
      align: 'right',
      render: (session) =>
        session.id === currentSessionId ? null : (
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
        ),
    },
  ]

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
        <Section title={t('sessions.title', { total })} contained={false}>
          <div className='mb-3 flex'>
            <FilterBar fields={filterFields} listing={listing} />
          </div>
          <DataView
            rows={sessions}
            columns={columns}
            card={{
              title: (session) => session.user_agent ?? t('sessions.unknown_device'),
              subtitle: (session) => session.ip_address ?? t('sessions.unknown_ip'),
              footer: (session) => formatDateTime(session.created_at),
            }}
            getKey={(session) => session.id}
            view={SESSION_VIEW}
            loading={isSessionsLoading}
            sort={listing.state.sort}
            onSortChange={listing.setSort}
            emptyLabel={narrowed ? t('sessions.empty_filtered') : t('sessions.empty')}
            emptyAction={
              narrowed ? (
                <Button variant='outline' onClick={listing.clearFilters}>
                  {t('sessions.show_all')}
                </Button>
              ) : undefined
            }
          />
          {pagination && (
            <div className='mt-3 px-1'>
              <PaginationBar pagination={pagination} onPageChange={listing.setPage} />
            </div>
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
