import { Bot, MailCheck, MailX, Plus } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/kit/button'
import { IconTile, ListingPage, Pill, Squircle, StatusDot } from '@/components/kit'
import type {
  CardSpec,
  Column,
  PagedListing,
  PaginationMetadata,
} from '@/components/kit'
import { isServiceAccount } from '@/utils'
import { formatRelative } from '@/utils/format-date'
import { roleRelationSource } from '@/api/role.relation'
import { Schemas } from '@/api/api.client'

import User = Schemas.User

export interface UserCounts {
  total: number
  enabled: number
  disabled: number
  verified: number
}

export interface PageUsersOverviewProps {
  users: User[]
  pagination: PaginationMetadata | undefined
  listing: PagedListing
  isLoading: boolean
  counts: UserCounts
  unverified: { total: number; names: string[] }
  userHref: (user: User) => string
  onCreate: () => void
}

const NAME_SEPARATOR = ', '
const TRUNCATION_MARK = '…'
const YES = 'true'
const NO = 'false'

export default function PageUsersOverview({
  users,
  pagination,
  listing,
  isLoading,
  counts,
  unverified,
  userHref,
  onCreate,
}: PageUsersOverviewProps) {
  const { t } = useTranslation('user')

  const displayName = (user: User) => {
    if (isServiceAccount(user)) return t('list.service_account_name')
    const full = [user.firstname, user.lastname].filter(Boolean).join(' ')
    return full || user.username
  }

  const typeLabel = (user: User) =>
    isServiceAccount(user) ? t('list.type.service') : t('list.type.user')

  const dimmed = (value: string | null | undefined) =>
    value ? (
      <span className='text-neutral-600 dark:text-neutral-400'>{value}</span>
    ) : (
      <span className='text-neutral-300 dark:text-neutral-600'>—</span>
    )

  const columns: Column<User>[] = [
    {
      key: 'user',
      header: t('list.columns.user'),
      render: (u) => (
        <span className='flex min-w-0 flex-col'>
          <span className='truncate text-neutral-900 dark:text-neutral-100'>{displayName(u)}</span>
          <span className='truncate font-mono-ui text-[11px] text-neutral-400 dark:text-neutral-500'>
            {u.username}
          </span>
        </span>
      ),
      sortKey: 'username',
      filters: [
        { kind: 'text', key: 'username', label: t('list.filter_fields.username') },
        {
          kind: 'relation',
          key: 'role_id',
          label: t('list.filter_fields.role'),
          relation: roleRelationSource,
        },
      ],
    },
    {
      key: 'firstname',
      header: t('list.columns.firstname'),
      render: (u) => dimmed(u.firstname),
      sortKey: 'firstname',
      filters: [{ kind: 'text', key: 'firstname', label: t('list.filter_fields.firstname') }],
    },
    {
      key: 'lastname',
      header: t('list.columns.lastname'),
      render: (u) => dimmed(u.lastname),
      sortKey: 'lastname',
      filters: [{ kind: 'text', key: 'lastname', label: t('list.filter_fields.lastname') }],
    },
    {
      key: 'email',
      header: t('list.columns.email'),
      render: (u) =>
        u.email ? (
          <span className='inline-flex items-center gap-1.5'>
            {isServiceAccount(u) ? null : u.email_verified ? (
              <MailCheck className='size-3.5 text-fk-success' />
            ) : (
              <MailX className='size-3.5 text-fk-amber' />
            )}
            <span className='text-neutral-600 dark:text-neutral-400'>{u.email}</span>
          </span>
        ) : (
          <span className='text-neutral-400 dark:text-neutral-500'>{t('list.no_email')}</span>
        ),
      sortKey: 'email',
      filters: [
        { kind: 'text', key: 'email', label: t('list.filter_fields.email') },
        { kind: 'boolean', key: 'email_verified', label: t('list.filter_fields.email_verified') },
      ],
    },
    {
      key: 'type',
      header: t('list.columns.type'),
      render: (u) => (
        <Pill tone={isServiceAccount(u) ? 'violet' : 'info'} mono>
          {typeLabel(u)}
        </Pill>
      ),
      filters: [
        { kind: 'boolean', key: 'service_account', label: t('list.filter_fields.service_account') },
      ],
    },
    {
      key: 'status',
      header: t('list.columns.status'),
      render: (u) => (
        <span className='inline-flex items-center gap-1.5 text-xs text-neutral-600 dark:text-neutral-400'>
          <StatusDot on={u.enabled} />
          {u.enabled ? t('list.status.enabled') : t('list.status.disabled')}
        </span>
      ),
      sortKey: 'enabled',
      filters: [{ kind: 'boolean', key: 'enabled', label: t('list.filter_fields.enabled') }],
    },
    {
      key: 'created',
      header: t('list.columns.created'),
      render: (u) => (
        <span className='tnum text-neutral-600 dark:text-neutral-400'>
          {formatRelative(u.created_at)}
        </span>
      ),
      sortKey: 'created_at',
      filters: [
        {
          kind: 'date-range',
          fromKey: 'created_from',
          toKey: 'created_to',
          label: t('list.filter_fields.created'),
        },
      ],
    },
    {
      key: 'updated',
      header: t('list.columns.updated'),
      render: (u) => (
        <span className='tnum text-neutral-600 dark:text-neutral-400'>
          {formatRelative(u.updated_at)}
        </span>
      ),
      sortKey: 'updated_at',
    },
  ]

  const card: CardSpec<User> = {
    avatar: (u) =>
      isServiceAccount(u) ? (
        <IconTile tone='violet'>
          <Bot className='size-4' strokeWidth={1.75} />
        </IconTile>
      ) : (
        <Squircle name={u.username} />
      ),
    title: (u) => displayName(u),
    subtitle: (u) => u.email || u.username,
    badges: (u) => (
      <>
        <Pill tone={u.enabled ? 'success' : 'neutral'}>
          <StatusDot on={u.enabled} />
          {u.enabled ? t('list.card.badge.enabled') : t('list.card.badge.disabled')}
        </Pill>
        <Pill tone={isServiceAccount(u) ? 'violet' : 'info'} mono>
          {typeLabel(u)}
        </Pill>
      </>
    ),
    flags: (u) => [
      { label: t('list.card.flags.enabled'), on: u.enabled },
      { label: t('list.card.flags.email_verified'), on: u.email_verified },
      { label: t('list.card.flags.no_required_action'), on: u.required_actions.length === 0 },
    ],
    footer: (u) => <span className='truncate'>{u.email || u.username}</span>,
  }

  const unverifiedNames =
    unverified.names.join(NAME_SEPARATOR) +
    (unverified.total > unverified.names.length ? TRUNCATION_MARK : '')

  const createButton = (
    <Button onClick={onCreate}>
      <Plus /> {t('list.create')}
    </Button>
  )

  return (
    <ListingPage
      title={t('list.title')}
      description={t('list.description')}
      loading={isLoading}
      actions={createButton}
      metrics={[
        {
          key: 'total',
          label: t('list.metrics.total.label'),
          value: counts.total,
          hint: t('list.metrics.total.hint'),
          series: [counts.total, counts.total],
          tone: 'info',
          filter: {},
        },
        {
          key: 'enabled',
          label: t('list.metrics.enabled.label'),
          value: counts.enabled,
          hint:
            counts.enabled > 0 && counts.total > 0
              ? t('list.metrics.enabled.hint', {
                  percent: ((counts.enabled / counts.total) * 100).toFixed(0),
                })
              : t('list.metrics.enabled.empty_hint'),
          series: [counts.enabled, counts.enabled],
          tone: 'success',
          filter: { enabled: YES },
        },
        {
          key: 'disabled',
          label: t('list.metrics.disabled.label'),
          value: counts.disabled,
          hint: t('list.metrics.disabled.hint'),
          series: [counts.disabled, counts.disabled],
          tone: 'amber',
          filter: { enabled: NO },
        },
        {
          key: 'verified',
          label: t('list.metrics.verified.label'),
          value: counts.verified,
          hint: t('list.metrics.verified.hint'),
          series: [counts.verified, counts.verified],
          tone: 'success',
          filter: { email_verified: YES },
        },
      ]}
      alerts={
        unverified.total
          ? [
              {
                tone: 'warn' as const,
                title: t('list.alerts.unverified_email.title', { count: unverified.total }),
                detail: t('list.alerts.unverified_email.detail', { names: unverifiedNames }),
                action: t('list.alerts.unverified_email.action'),
              },
            ]
          : []
      }
      paged={{ listing, pagination, search: { placeholder: t('list.search_placeholder') } }}
      rows={users}
      columns={columns}
      card={card}
      getKey={(u) => u.id}
      getHref={userHref}
      aggregates={{
        user: t('list.aggregates.count', { count: counts.total }),
        status: t('list.aggregates.enabled', { total: counts.enabled }),
      }}
      emptyLabel={t('list.empty.label')}
      emptyHint={t('list.empty.hint')}
      emptyAction={createButton}
    />
  )
}
