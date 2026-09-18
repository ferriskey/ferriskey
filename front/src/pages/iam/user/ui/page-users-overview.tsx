import { Bot, MailCheck, MailX, Plus } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/kit/button'
import { IconTile, ListingPage, Pill, Squircle, StatusDot } from '@/components/kit'
import type { CardSpec, Column } from '@/components/kit'
import { isServiceAccount } from '@/utils'
import { Schemas } from '@/api/api.client'

import User = Schemas.User
import { cumulativeSeries } from '@/utils/cumulative-series'

const QUERY_SYNTAX = 'username:adm*  email_verified:false  enabled:true'

export interface PageUsersOverviewProps {
  users: User[]
  isLoading: boolean
  userHref: (user: User) => string
  onCreate: () => void
}

export default function PageUsersOverview({
  users,
  isLoading,
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
      sortValue: (u) => displayName(u),
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
      sortValue: (u) => u.email ?? '',
    },
    {
      key: 'type',
      header: t('list.columns.type'),
      render: (u) => (
        <Pill tone={isServiceAccount(u) ? 'violet' : 'info'} mono>
          {typeLabel(u)}
        </Pill>
      ),
      sortValue: (u) => typeLabel(u),
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

  const enabled = users.filter((u) => u.enabled)
  const disabled = users.filter((u) => !u.enabled)
  const verified = users.filter((u) => u.email_verified)
  const unverified = users.filter((u) => !u.email_verified && !isServiceAccount(u))

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
          value: users.length,
          hint: t('list.metrics.total.hint'),
          series: cumulativeSeries(users.map((r) => r.created_at)),
          tone: 'info',
        },
        {
          key: 'enabled',
          label: t('list.metrics.enabled.label'),
          value: enabled.length,
          hint:
            enabled.length > 0 && users.length > 0
              ? t('list.metrics.enabled.hint', {
                  percent: ((enabled.length / users.length) * 100).toFixed(0),
                })
              : t('list.metrics.enabled.empty_hint'),
          series: cumulativeSeries(enabled.map((r) => r.created_at)),
          tone: 'success',
        },
        {
          key: 'disabled',
          label: t('list.metrics.disabled.label'),
          value: disabled.length,
          hint: t('list.metrics.disabled.hint'),
          series: cumulativeSeries(disabled.map((r) => r.created_at)),
          tone: 'amber',
        },
        {
          key: 'verified',
          label: t('list.metrics.verified.label'),
          value: verified.length,
          hint: t('list.metrics.verified.hint'),
          series: cumulativeSeries(verified.map((r) => r.created_at)),
          tone: 'success',
        },
      ]}
      alerts={
        unverified.length
          ? [
              {
                tone: 'warn' as const,
                title: t('list.alerts.unverified_email.title', { count: unverified.length }),
                detail: t('list.alerts.unverified_email.detail', {
                  names: unverified.map((u) => u.username).join(', '),
                }),
                action: t('list.alerts.unverified_email.action'),
              },
            ]
          : []
      }
      filters={[
        { key: 'users', label: t('list.filters.users'), predicate: (u) => !isServiceAccount(u) },
        {
          key: 'service',
          label: t('list.filters.service_accounts'),
          predicate: isServiceAccount,
        },
        {
          key: 'unverified',
          label: t('list.filters.unverified'),
          predicate: (u) => !u.email_verified && !isServiceAccount(u),
        },
      ]}
      searchPlaceholder={t('list.search_placeholder')}
      querySyntax={QUERY_SYNTAX}
      searchIn={(u) =>
        `${u.username} ${u.email ?? ''} ${u.firstname ?? ''} ${u.lastname ?? ''}`
      }
      rows={users}
      columns={columns}
      card={card}
      getKey={(u) => u.id}
      getHref={userHref}
      aggregates={{
        user: t('list.aggregates.count', { count: users.length }),
        status: t('list.aggregates.enabled', { total: enabled.length }),
      }}
      emptyLabel={t('list.empty.label')}
      emptyHint={t('list.empty.hint')}
      emptyAction={createButton}
    />
  )
}
