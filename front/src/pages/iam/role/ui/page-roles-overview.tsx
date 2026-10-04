import { Plus, Shield, ShieldCheck } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/kit/button'
import { ListingPage, IconTile, Pill } from '@/components/kit'
import type {
  CardSpec,
  Column,
  FilterField,
  PagedListing,
  PaginationMetadata,
} from '@/components/kit'
import { clientRelationSource } from '@/api/client.relation'
import { Schemas } from '@/api/api.client'
import { formatRelative } from '@/utils/format-date'

import Role = Schemas.Role
import { roleScopeLabelKey } from '../role-scope'

export interface RoleCounts {
  total: number
  realm: number
  client: number
  granting: number
}

export interface RolesWithoutPermissions {
  total: number
  names: string[]
}

export interface PageRolesOverviewProps {
  roles: Role[]
  pagination: PaginationMetadata | undefined
  listing: PagedListing
  isLoading: boolean
  counts: RoleCounts
  withoutPermissions: RolesWithoutPermissions
  roleHref: (role: Role) => string
  onCreate: () => void
}

const NAME_SEPARATOR = ', '
const TRUNCATION_MARK = '…'

const isClientRole = (role: Role) => Boolean(role.client_id)

export default function PageRolesOverview({
  roles,
  pagination,
  listing,
  isLoading,
  counts,
  withoutPermissions,
  roleHref,
  onCreate,
}: PageRolesOverviewProps) {
  const { t } = useTranslation('role')

  const columns: Column<Role>[] = [
    {
      key: 'name',
      header: t('list.columns.name'),
      render: (r) => r.name,
      sortKey: 'name',
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
    },
    {
      key: 'created',
      header: t('list.columns.created'),
      render: (r) => (
        <span className='tnum text-neutral-600 dark:text-neutral-400'>
          {formatRelative(r.created_at)}
        </span>
      ),
      sortKey: 'created_at',
    },
    {
      key: 'updated',
      header: t('list.columns.updated'),
      render: (r) => (
        <span className='tnum text-neutral-600 dark:text-neutral-400'>
          {formatRelative(r.updated_at)}
        </span>
      ),
      sortKey: 'updated_at',
    },
  ]

  const filterFields: FilterField[] = [
    { kind: 'text', key: 'name', label: t('list.filter_fields.name') },
    { kind: 'text', key: 'description', label: t('list.filter_fields.description') },
    { kind: 'boolean', key: 'require_mfa', label: t('list.filter_fields.require_mfa') },
    {
      kind: 'enum',
      key: 'scope',
      label: t('list.filter_fields.scope'),
      options: [
        { value: 'realm', label: t('list.filters.realm') },
        { value: 'client', label: t('list.filters.client') },
      ],
    },
    {
      kind: 'boolean',
      key: 'has_permissions',
      label: t('list.filter_fields.has_permissions'),
    },
    {
      kind: 'relation',
      key: 'client_id',
      label: t('list.filter_fields.client'),
      relation: clientRelationSource,
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

  const withoutPermissionsNames =
    withoutPermissions.names.join(NAME_SEPARATOR) +
    (withoutPermissions.total > withoutPermissions.names.length ? TRUNCATION_MARK : '')

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
        },
        {
          key: 'realm',
          label: t('list.metrics.realm.label'),
          value: counts.realm,
          hint:
            counts.realm > 0 && counts.total > 0
              ? t('list.metrics.realm.hint', {
                  percent: ((counts.realm / counts.total) * 100).toFixed(0),
                })
              : t('list.metrics.realm.empty_hint'),
          series: [counts.realm, counts.realm],
          tone: 'info',
        },
        {
          key: 'client',
          label: t('list.metrics.client.label'),
          value: counts.client,
          hint: t('list.metrics.client.hint'),
          series: [counts.client, counts.client],
          tone: 'violet',
        },
        {
          key: 'granting',
          label: t('list.metrics.granting.label'),
          value: counts.granting,
          hint: t('list.metrics.granting.hint'),
          series: [counts.granting, counts.granting],
          tone: 'success',
        },
      ]}
      alerts={
        withoutPermissions.total
          ? [
              {
                tone: 'warn' as const,
                title: t('list.alerts.without_permissions.title', {
                  count: withoutPermissions.total,
                }),
                detail: t('list.alerts.without_permissions.detail', {
                  names: withoutPermissionsNames,
                }),
                action: t('list.alerts.without_permissions.action'),
              },
            ]
          : []
      }
      paged={{ listing, pagination, filterFields }}
      rows={roles}
      columns={columns}
      card={card}
      getKey={(r) => r.id}
      getHref={roleHref}
      aggregates={{
        name: t('list.count', { count: counts.total }),
      }}
      emptyLabel={t('list.empty.label')}
      emptyHint={t('list.empty.hint')}
      emptyAction={createButton}
    />
  )
}
