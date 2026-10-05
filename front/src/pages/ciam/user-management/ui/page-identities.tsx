import { MailCheck, MailX, Plus, UserCog } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/kit/button'
import { ListingPage, Pill, Squircle, StatusDot } from '@/components/kit'
import type {
  CardSpec,
  Column,
  PagedListing,
  PaginationMetadata,
} from '@/components/kit'
import { formatRelative } from '@/utils/format-date'
import { roleRelationSource } from '@/api/role.relation'
import { Schemas } from '@/api/api.client'

import User = Schemas.User

export interface IdentityCounts {
  total: number
  verified: number
  disabled: number
  active: number
}

export interface PageIdentitiesProps {
  identities: User[]
  pagination: PaginationMetadata | undefined
  listing: PagedListing
  isLoading: boolean
  counts: IdentityCounts
  identityHref: (identity: User) => string
  onCreate: () => void
  onReviewPending: () => void
}

const displayName = (user: User) => {
  const full = [user.firstname, user.lastname].filter(Boolean).join(' ')
  return full || user.username
}

const pendingCount = (user: User) => user.required_actions.length

const NAME_SEPARATOR = ', '
const YES = 'true'
const NO = 'false'

export default function PageIdentities({
  identities,
  pagination,
  listing,
  isLoading,
  counts,
  identityHref,
  onCreate,
  onReviewPending,
}: PageIdentitiesProps) {
  const { t } = useTranslation('console')

  const columns: Column<User>[] = [
    {
      key: 'identity',
      header: t('identities.list.columns.identity'),
      render: (u) => (
        <span className='flex min-w-0 flex-col'>
          <span className='truncate text-neutral-900 dark:text-neutral-100'>{displayName(u)}</span>
          <span className='truncate font-mono-ui text-[11px] text-neutral-400 dark:text-neutral-500'>
            {u.username}
          </span>
        </span>
      ),
      sortKey: 'username',
    },
    {
      key: 'email',
      header: t('identities.list.columns.email'),
      render: (u) =>
        u.email ? (
          <span className='inline-flex items-center gap-1.5'>
            {u.email_verified ? (
              <MailCheck className='size-3.5 text-fk-success' />
            ) : (
              <MailX className='size-3.5 text-fk-amber' />
            )}
            <span className='text-neutral-600 dark:text-neutral-400'>{u.email}</span>
          </span>
        ) : (
          <span className='text-neutral-400 dark:text-neutral-500'>
            {t('identities.list.no_email')}
          </span>
        ),
      sortKey: 'email',
      filters: [
        {
          kind: 'boolean',
          key: 'email_verified',
          label: t('identities.list.filter_fields.email_verified'),
        },
      ],
    },
    {
      key: 'status',
      header: t('identities.list.columns.status'),
      render: (u) => {
        if (!u.enabled) {
          return (
            <span className='inline-flex items-center gap-1.5 text-xs text-neutral-600 dark:text-neutral-400'>
              <StatusDot on={false} />
              {t('identities.list.status.disabled')}
            </span>
          )
        }
        const pending = pendingCount(u)
        if (pending > 0) {
          return (
            <Pill tone='amber'>
              <UserCog className='size-3' strokeWidth={1.75} />
              {t('identities.list.status.pending', { count: pending })}
            </Pill>
          )
        }
        return (
          <span className='inline-flex items-center gap-1.5 text-xs text-neutral-600 dark:text-neutral-400'>
            <StatusDot on />
            {t('identities.list.status.active')}
          </span>
        )
      },
      sortKey: 'enabled',
      filters: [
        { kind: 'boolean', key: 'enabled', label: t('identities.list.filter_fields.enabled') },
      ],
    },
    {
      key: 'created',
      header: t('identities.list.columns.created'),
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
          label: t('identities.list.filter_fields.created'),
        },
      ],
    },
    {
      key: 'roles',
      header: t('identities.list.columns.roles'),
      align: 'right',
      render: (u) => {
        const count = u.roles?.length ?? 0
        return count > 0 ? (
          <span className='tnum text-neutral-600 dark:text-neutral-400'>{count}</span>
        ) : (
          <span className='tnum text-neutral-300 dark:text-neutral-600'>0</span>
        )
      },
      filters: [
        {
          kind: 'relation',
          key: 'role_id',
          label: t('identities.list.filter_fields.role'),
          relation: roleRelationSource,
        },
      ],
    },
  ]

  const card: CardSpec<User> = {
    avatar: (u) => <Squircle name={u.username} />,
    title: (u) => displayName(u),
    subtitle: (u) => u.email || u.username,
    badges: (u) => (
      <>
        <Pill tone={u.enabled ? 'success' : 'neutral'}>
          <StatusDot on={u.enabled} />
          {u.enabled ? t('identities.list.card.active') : t('identities.list.card.disabled')}
        </Pill>
        <Pill tone={u.email_verified ? 'info' : 'amber'} mono>
          {u.email_verified
            ? t('identities.list.card.email_verified')
            : t('identities.list.card.email_unverified')}
        </Pill>
        {pendingCount(u) > 0 && (
          <Pill tone='amber'>
            {t('identities.list.card.pending', { count: pendingCount(u) })}
          </Pill>
        )}
      </>
    ),
    flags: (u) => [
      { label: t('identities.list.card.flags.can_sign_in'), on: u.enabled },
      { label: t('identities.list.card.flags.email_verified'), on: u.email_verified },
      {
        label: t('identities.list.card.flags.nothing_required'),
        on: pendingCount(u) === 0,
      },
    ],
    footer: (u) => (
      <span className='truncate'>
        {t('identities.list.card.signed_up', { when: formatRelative(u.created_at) })}
      </span>
    ),
  }

  const pending = identities.filter((u) => pendingCount(u) > 0)

  const createButton = (
    <Button onClick={onCreate}>
      <Plus /> {t('identities.list.create')}
    </Button>
  )

  return (
    <ListingPage
      title={t('identities.list.title')}
      description={t('identities.list.description')}
      loading={isLoading}
      actions={createButton}
      metrics={[
        {
          key: 'total',
          label: t('identities.list.metrics.total.label'),
          value: counts.total,
          hint: t('identities.list.metrics.total.hint', { count: counts.total }),
          series: [counts.total, counts.total],
          filter: {},
        },
        {
          key: 'verified',
          label: t('identities.list.metrics.verified.label'),
          value: counts.verified,
          hint:
            counts.verified > 0 && counts.total > 0
              ? t('identities.list.metrics.verified.hint', {
                  percent: ((counts.verified / counts.total) * 100).toFixed(0),
                })
              : t('identities.list.metrics.verified.empty_hint'),
          series: [counts.verified, counts.verified],
          filter: { email_verified: YES },
        },
        {
          key: 'pending',
          label: t('identities.list.metrics.pending.label'),
          value: pending.length,
          hint: t('identities.list.metrics.pending.hint'),
        },
        {
          key: 'disabled',
          label: t('identities.list.metrics.disabled.label'),
          value: counts.disabled,
          hint: t('identities.list.metrics.disabled.hint'),
          series: [counts.disabled, counts.disabled],
          filter: { enabled: NO },
        },
      ]}
      alerts={
        pending.length
          ? [
              {
                tone: 'warn' as const,
                title: t('identities.list.alerts.pending.title', { count: pending.length }),
                detail: t('identities.list.alerts.pending.detail', {
                  names: pending.map((u) => u.username).join(NAME_SEPARATOR),
                }),
                action: t('identities.list.alerts.pending.action'),
                onAction: onReviewPending,
              },
            ]
          : []
      }
      paged={{
        listing,
        pagination,
        search: { placeholder: t('identities.list.search_placeholder') },
      }}
      rows={identities}
      columns={columns}
      card={card}
      getKey={(u) => u.id}
      getHref={identityHref}
      aggregates={{
        identity: t('identities.list.aggregates.identities', { count: counts.total }),
        status: t('identities.list.aggregates.active', { total: counts.active }),
      }}
      emptyLabel={t('identities.list.empty.label')}
      emptyHint={t('identities.list.empty.hint')}
      emptyAction={createButton}
    />
  )
}
