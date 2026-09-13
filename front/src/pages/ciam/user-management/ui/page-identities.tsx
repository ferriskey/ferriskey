import { MailCheck, MailX, Plus, UserCog } from 'lucide-react'
import { Button } from '@/components/kit/button'
import { ListingPage, Pill, Squircle, StatusDot } from '@/components/kit'
import type { CardSpec, Column } from '@/components/kit'
import { formatRelative } from '@/utils/format-date'
import { Schemas } from '@/api/api.client'

import User = Schemas.User

export interface PageIdentitiesProps {
  identities: User[]
  isLoading: boolean
  identityHref: (identity: User) => string
  onCreate: () => void
  onReviewPending: () => void
}

const displayName = (user: User) => {
  const full = [user.firstname, user.lastname].filter(Boolean).join(' ')
  return full || user.username
}

const pendingCount = (user: User) => user.required_actions.length

export default function PageIdentities({
  identities,
  isLoading,
  identityHref,
  onCreate,
  onReviewPending,
}: PageIdentitiesProps) {
  const columns: Column<User>[] = [
    {
      key: 'identity',
      header: 'Identity',
      render: (u) => (
        <span className='flex min-w-0 flex-col'>
          <span className='truncate text-neutral-900 dark:text-neutral-100'>{displayName(u)}</span>
          <span className='truncate font-mono-ui text-[11px] text-neutral-400 dark:text-neutral-500'>
            {u.username}
          </span>
        </span>
      ),
      sortValue: (u) => displayName(u),
    },
    {
      key: 'email',
      header: 'Email',
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
          <span className='text-neutral-400 dark:text-neutral-500'>no email</span>
        ),
      sortValue: (u) => u.email ?? '',
    },
    {
      key: 'status',
      header: 'Status',
      render: (u) => {
        if (!u.enabled) {
          return (
            <span className='inline-flex items-center gap-1.5 text-xs text-neutral-600 dark:text-neutral-400'>
              <StatusDot on={false} />
              Disabled
            </span>
          )
        }
        const pending = pendingCount(u)
        if (pending > 0) {
          return (
            <Pill tone='amber'>
              <UserCog className='size-3' strokeWidth={1.75} />
              {pending} action{pending > 1 ? 's' : ''}
            </Pill>
          )
        }
        return (
          <span className='inline-flex items-center gap-1.5 text-xs text-neutral-600 dark:text-neutral-400'>
            <StatusDot on />
            Active
          </span>
        )
      },
      sortValue: (u) => (!u.enabled ? 0 : pendingCount(u) > 0 ? 1 : 2),
    },
    {
      key: 'created',
      header: 'Signed up',
      render: (u) => (
        <span className='tnum text-neutral-600 dark:text-neutral-400'>
          {formatRelative(u.created_at)}
        </span>
      ),
      sortValue: (u) => u.created_at,
    },
    {
      key: 'roles',
      header: 'Roles',
      align: 'right',
      render: (u) => {
        const count = u.roles?.length ?? 0
        return count > 0 ? (
          <span className='tnum text-neutral-600 dark:text-neutral-400'>{count}</span>
        ) : (
          <span className='tnum text-neutral-300 dark:text-neutral-600'>0</span>
        )
      },
      sortValue: (u) => u.roles?.length ?? 0,
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
          {u.enabled ? 'active' : 'disabled'}
        </Pill>
        <Pill tone={u.email_verified ? 'info' : 'amber'} mono>
          {u.email_verified ? 'email verified' : 'email unverified'}
        </Pill>
        {pendingCount(u) > 0 && (
          <Pill tone='amber'>
            {pendingCount(u)} pending action{pendingCount(u) > 1 ? 's' : ''}
          </Pill>
        )}
      </>
    ),
    flags: (u) => [
      { label: 'Can sign in', on: u.enabled },
      { label: 'Email verified', on: u.email_verified },
      { label: 'Nothing required at next sign-in', on: pendingCount(u) === 0 },
    ],
    footer: (u) => <span className='truncate'>Signed up {formatRelative(u.created_at)}</span>,
  }

  const active = identities.filter((u) => u.enabled)
  const verified = identities.filter((u) => u.email_verified)
  const pending = identities.filter((u) => pendingCount(u) > 0)
  const disabled = identities.filter((u) => !u.enabled)

  const createButton = (
    <Button onClick={onCreate}>
      <Plus /> New identity
    </Button>
  )

  return (
    <ListingPage
      title='Identities'
      description='Customer accounts of this realm. Search them, review their sign-in state and what they still have to do.'
      loading={isLoading}
      actions={createButton}
      metrics={[
        {
          key: 'total',
          label: 'Total',
          value: identities.length,
          hint: `identit${identities.length === 1 ? 'y' : 'ies'}`,
        },
        {
          key: 'verified',
          label: 'Email verified',
          value: verified.length,
          hint:
            verified.length > 0 && identities.length > 0
              ? `${((verified.length / identities.length) * 100).toFixed(0)}% of total`
              : 'none verified',
        },
        {
          key: 'pending',
          label: 'Pending actions',
          value: pending.length,
          hint: 'blocked at next sign-in',
        },
        {
          key: 'disabled',
          label: 'Disabled',
          value: disabled.length,
          hint: 'cannot sign in',
        },
      ]}
      alerts={
        pending.length
          ? [
              {
                tone: 'warn' as const,
                title: `${pending.length} identit${pending.length > 1 ? 'ies have' : 'y has'} an action pending at next sign-in`,
                detail: `${pending.map((u) => u.username).join(', ')} — they cannot complete a sign-in until it is done.`,
                action: 'Review',
                onAction: onReviewPending,
              },
            ]
          : []
      }
      filters={[
        { key: 'active', label: 'Active', predicate: (u) => u.enabled },
        { key: 'unverified', label: 'Unverified email', predicate: (u) => !u.email_verified },
        { key: 'pending', label: 'Pending actions', predicate: (u) => pendingCount(u) > 0 },
        { key: 'disabled', label: 'Disabled', predicate: (u) => !u.enabled },
      ]}
      searchPlaceholder='Filter by name, email or username…'
      querySyntax='username:ada*  email_verified:false  enabled:true'
      searchIn={(u) => `${u.username} ${u.email ?? ''} ${u.firstname ?? ''} ${u.lastname ?? ''}`}
      rows={identities}
      columns={columns}
      card={card}
      getKey={(u) => u.id}
      getHref={identityHref}
      aggregates={{
        identity: `${identities.length} identit${identities.length === 1 ? 'y' : 'ies'}`,
        status: `${active.length} active`,
      }}
      emptyLabel='No identity'
      emptyHint='An identity is a customer account of this realm. Service accounts belong to applications and are listed there.'
      emptyAction={createButton}
    />
  )
}
