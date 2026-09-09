import { Bot, MailCheck, MailX, Plus } from 'lucide-react'
import { Button } from '@/components/kit/button'
import { IconTile, ListingPage, Pill, Squircle, StatusDot } from '@/components/kit'
import type { CardSpec, Column } from '@/components/kit'
import { isServiceAccount } from '@/utils'
import { Schemas } from '@/api/api.client'

import User = Schemas.User
import { cumulativeSeries } from '@/next/shared/cumulative-series'

export interface PageUsersOverviewProps {
  users: User[]
  isLoading: boolean
  userHref: (user: User) => string
  onCreate: () => void
}

function displayName(user: User) {
  if (isServiceAccount(user)) return 'Service Account'
  const full = [user.firstname, user.lastname].filter(Boolean).join(' ')
  return full || user.username
}

export default function PageUsersOverview({
  users,
  isLoading,
  userHref,
  onCreate,
}: PageUsersOverviewProps) {
  const columns: Column<User>[] = [
    {
      key: 'user',
      header: 'User',
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
            {isServiceAccount(u) ? null : u.email_verified ? (
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
      key: 'type',
      header: 'Type',
      render: (u) => (
        <Pill tone={isServiceAccount(u) ? 'violet' : 'info'} mono>
          {isServiceAccount(u) ? 'service account' : 'user account'}
        </Pill>
      ),
      sortValue: (u) => (isServiceAccount(u) ? 'service account' : 'user account'),
    },
    {
      key: 'status',
      header: 'Status',
      render: (u) => (
        <span className='inline-flex items-center gap-1.5 text-xs text-neutral-600 dark:text-neutral-400'>
          <StatusDot on={u.enabled} />
          {u.enabled ? 'Enabled' : 'Disabled'}
        </span>
      ),
      sortValue: (u) => (u.enabled ? 1 : 0),
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
          {u.enabled ? 'enabled' : 'disabled'}
        </Pill>
        <Pill tone={isServiceAccount(u) ? 'violet' : 'info'} mono>
          {isServiceAccount(u) ? 'service account' : 'user account'}
        </Pill>
      </>
    ),
    flags: (u) => [
      { label: 'Account enabled', on: u.enabled },
      { label: 'Email verified', on: u.email_verified },
      { label: 'No action required at next sign-in', on: u.required_actions.length === 0 },
    ],
    footer: (u) => <span className='truncate'>{u.email || u.username}</span>,
  }

  const enabled = users.filter((u) => u.enabled)
  const disabled = users.filter((u) => !u.enabled)
  const verified = users.filter((u) => u.email_verified)
  const unverified = users.filter((u) => !u.email_verified && !isServiceAccount(u))

  const createButton = (
    <Button onClick={onCreate}>
      <Plus /> New user
    </Button>
  )

  return (
    <ListingPage
      title='Users'
      description='Accounts of this realm, their type and their authentication state.'
      loading={isLoading}
      actions={createButton}
      metrics={[
        { key: 'total', label: 'Total', value: users.length, hint: 'users', series: cumulativeSeries(users.map((r) => r.created_at)), tone: 'info' },
        {
          key: 'enabled',
          label: 'Enabled users',
          value: enabled.length,
          hint:
            enabled.length > 0 && users.length > 0
              ? `${((enabled.length / users.length) * 100).toFixed(0)}% active`
              : 'No enabled users',
          series: cumulativeSeries(enabled.map((r) => r.created_at)),
          tone: 'success',
        },
        {
          key: 'disabled',
          label: 'Disabled users',
          value: disabled.length,
          hint: 'cannot authenticate',
          series: cumulativeSeries(disabled.map((r) => r.created_at)),
          tone: 'amber',
        },
        {
          key: 'verified',
          label: 'Verified users',
          value: verified.length,
          hint: 'email verified',
          series: cumulativeSeries(verified.map((r) => r.created_at)),
          tone: 'success',
        },
      ]}
      alerts={
        unverified.length
          ? [
              {
                tone: 'warn' as const,
                title: `${unverified.length} account${unverified.length > 1 ? 's have' : ' has'} an unverified email`,
                detail: `${unverified.map((u) => u.username).join(', ')} — password recovery will not reach them.`,
                action: 'Review',
              },
            ]
          : []
      }
      filters={[
        { key: 'users', label: 'Users', predicate: (u) => !isServiceAccount(u) },
        { key: 'service', label: 'Service accounts', predicate: isServiceAccount },
        {
          key: 'unverified',
          label: 'Unverified email',
          predicate: (u) => !u.email_verified && !isServiceAccount(u),
        },
      ]}
      searchPlaceholder='Filter by name or email…'
      querySyntax='username:adm*  email_verified:false  enabled:true'
      searchIn={(u) =>
        `${u.username} ${u.email ?? ''} ${u.firstname ?? ''} ${u.lastname ?? ''}`
      }
      rows={users}
      columns={columns}
      card={card}
      getKey={(u) => u.id}
      getHref={userHref}
      aggregates={{
        user: `${users.length} user${users.length !== 1 ? 's' : ''}`,
        status: `${enabled.length} enabled`,
      }}
      emptyLabel='No user'
      emptyHint='An account authenticates against this realm; a service account belongs to a client.'
      emptyAction={createButton}
    />
  )
}
