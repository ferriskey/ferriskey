import { Plus, Shield, ShieldCheck } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/kit/button'
import { ListingPage, IconTile, Pill } from '@/components/kit'
import type { CardSpec, Column } from '@/components/kit'
import { Schemas } from '@/api/api.client'

import Role = Schemas.Role
import { cumulativeSeries } from '@/utils/cumulative-series'
import { roleScopeLabelKey } from '../role-scope'

const QUERY_SYNTAX = 'name:realm*  scope:client  permissions:0'

export interface PageRolesOverviewProps {
  roles: Role[]
  isLoading: boolean
  roleHref: (role: Role) => string
  onCreate: () => void
}

const isClientRole = (role: Role) => Boolean(role.client_id)

export default function PageRolesOverview({
  roles,
  isLoading,
  roleHref,
  onCreate,
}: PageRolesOverviewProps) {
  const { t } = useTranslation('role')

  const columns: Column<Role>[] = [
    {
      key: 'name',
      header: t('list.columns.name'),
      render: (r) => r.name,
      sortValue: (r) => r.name,
    },
    {
      key: 'description',
      header: t('list.columns.description'),
      render: (r) =>
        r.description ? (
          <span className='text-neutral-600 dark:text-neutral-400'>{r.description}</span>
        ) : (
          <span className='font-mono-ui text-xs text-neutral-400 dark:text-neutral-500'>
            {t('role.identifier', { id: r.id })}
          </span>
        ),
    },
    {
      key: 'scope',
      header: t('list.columns.scope'),
      render: (r) => (
        <Pill tone={isClientRole(r) ? 'violet' : 'info'} mono>
          {t(roleScopeLabelKey(isClientRole(r)))}
        </Pill>
      ),
      sortValue: (r) => t(roleScopeLabelKey(isClientRole(r))),
    },
    {
      key: 'permissions',
      header: t('list.columns.permissions'),
      align: 'right',
      render: (r) =>
        r.permissions.length > 0 ? (
          <span className='tnum text-neutral-600 dark:text-neutral-400'>{r.permissions.length}</span>
        ) : (
          <span className='tnum text-neutral-300 dark:text-neutral-600'>0</span>
        ),
      sortValue: (r) => r.permissions.length,
    },
  ]

  const card: CardSpec<Role> = {
    avatar: (r) => (
      <IconTile tone={isClientRole(r) ? 'violet' : 'info'}>
        {isClientRole(r) ? (
          <ShieldCheck className='size-4' strokeWidth={1.75} />
        ) : (
          <Shield className='size-4' strokeWidth={1.75} />
        )}
      </IconTile>
    ),
    title: (r) => r.name,
    subtitle: (r) => t(roleScopeLabelKey(isClientRole(r))),
    badges: (r) => (
      <>
        <Pill tone={isClientRole(r) ? 'violet' : 'info'} mono>
          {t(roleScopeLabelKey(isClientRole(r)))}
        </Pill>
        <Pill tone={r.permissions.length > 0 ? 'success' : 'amber'}>
          {t('role.permission_count', { count: r.permissions.length })}
        </Pill>
      </>
    ),
    flags: (r) => [
      { label: t('list.card.flags.grants'), on: r.permissions.length > 0 },
      { label: t('list.card.flags.client_scoped'), on: isClientRole(r) },
    ],
    footer: (r) => (
      <span className='truncate'>{r.description || t('role.identifier', { id: r.id })}</span>
    ),
  }

  const realmRoles = roles.filter((r) => !isClientRole(r))
  const clientRoles = roles.filter(isClientRole)
  const withPermissions = roles.filter((r) => r.permissions.length > 0)
  const empty = roles.filter((r) => r.permissions.length === 0)

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
          value: roles.length,
          hint: t('list.metrics.total.hint'),
          series: cumulativeSeries(roles.map((r) => r.created_at)),
          tone: 'info',
        },
        {
          key: 'realm',
          label: t('list.metrics.realm.label'),
          value: realmRoles.length,
          hint:
            realmRoles.length > 0 && roles.length > 0
              ? t('list.metrics.realm.hint', {
                  percent: ((realmRoles.length / roles.length) * 100).toFixed(0),
                })
              : t('list.metrics.realm.empty_hint'),
          series: cumulativeSeries(realmRoles.map((r) => r.created_at)),
          tone: 'info',
        },
        {
          key: 'client',
          label: t('list.metrics.client.label'),
          value: clientRoles.length,
          hint: t('list.metrics.client.hint'),
          series: cumulativeSeries(clientRoles.map((r) => r.created_at)),
          tone: 'violet',
        },
        {
          key: 'granting',
          label: t('list.metrics.granting.label'),
          value: withPermissions.length,
          hint: t('list.metrics.granting.hint'),
          series: cumulativeSeries(withPermissions.map((r) => r.created_at)),
          tone: 'success',
        },
      ]}
      alerts={
        empty.length
          ? [
              {
                tone: 'warn' as const,
                title: t('list.alerts.without_permissions.title', { count: empty.length }),
                detail: t('list.alerts.without_permissions.detail', {
                  names: empty.map((r) => r.name).join(', '),
                }),
                action: t('list.alerts.without_permissions.action'),
              },
            ]
          : []
      }
      filters={[
        { key: 'realm', label: t('list.filters.realm'), predicate: (r) => !isClientRole(r) },
        { key: 'client', label: t('list.filters.client'), predicate: isClientRole },
        {
          key: 'empty',
          label: t('list.filters.without_permissions'),
          predicate: (r) => r.permissions.length === 0,
        },
      ]}
      searchPlaceholder={t('list.search_placeholder')}
      querySyntax={QUERY_SYNTAX}
      searchIn={(r) => `${r.name} ${r.description ?? ''}`}
      rows={roles}
      columns={columns}
      card={card}
      getKey={(r) => r.id}
      getHref={roleHref}
      aggregates={{
        name: t('list.count', { count: roles.length }),
        permissions: roles.reduce((n, r) => n + r.permissions.length, 0),
      }}
      emptyLabel={t('list.empty.label')}
      emptyHint={t('list.empty.hint')}
      emptyAction={createButton}
    />
  )
}
