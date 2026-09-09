import { Plus, Shield, ShieldCheck } from 'lucide-react'
import { Button } from '@/components/kit/button'
import { ListingPage, IconTile, Pill } from '@/components/kit'
import type { CardSpec, Column } from '@/components/kit'
import { Schemas } from '@/api/api.client'

import Role = Schemas.Role

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
  const columns: Column<Role>[] = [
    {
      key: 'name',
      header: 'Role',
      render: (r) => r.name,
      sortValue: (r) => r.name,
    },
    {
      key: 'description',
      header: 'Description',
      render: (r) =>
        r.description ? (
          <span className='text-neutral-600 dark:text-neutral-400'>{r.description}</span>
        ) : (
          <span className='font-mono-ui text-xs text-neutral-400 dark:text-neutral-500'>
            role_id: {r.id}
          </span>
        ),
    },
    {
      key: 'scope',
      header: 'Scope',
      render: (r) => (
        <Pill tone={isClientRole(r) ? 'violet' : 'info'} mono>
          {isClientRole(r) ? 'client' : 'realm'}
        </Pill>
      ),
      sortValue: (r) => (isClientRole(r) ? 'client' : 'realm'),
    },
    {
      key: 'permissions',
      header: 'Permissions',
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
    subtitle: (r) => (isClientRole(r) ? 'client' : 'realm'),
    badges: (r) => (
      <>
        <Pill tone={isClientRole(r) ? 'violet' : 'info'} mono>
          {isClientRole(r) ? 'client' : 'realm'}
        </Pill>
        <Pill tone={r.permissions.length > 0 ? 'success' : 'amber'}>
          {r.permissions.length} permission{r.permissions.length !== 1 ? 's' : ''}
        </Pill>
      </>
    ),
    flags: (r) => [
      { label: 'Grants at least one permission', on: r.permissions.length > 0 },
      { label: 'Scoped to a client', on: isClientRole(r) },
    ],
    footer: (r) => <span className='truncate'>{r.description || `role_id: ${r.id}`}</span>,
  }

  const realmRoles = roles.filter((r) => !isClientRole(r))
  const clientRoles = roles.filter(isClientRole)
  const withPermissions = roles.filter((r) => r.permissions.length > 0)
  const empty = roles.filter((r) => r.permissions.length === 0)

  const createButton = (
    <Button onClick={onCreate}>
      <Plus /> New role
    </Button>
  )

  return (
    <ListingPage
      title='Roles'
      description='Permissions and policies applicable to the accounts of this realm.'
      loading={isLoading}
      actions={createButton}
      metrics={[
        { key: 'total', label: 'Total', value: roles.length, hint: 'roles' },
        {
          key: 'realm',
          label: 'Realm roles',
          value: realmRoles.length,
          hint:
            realmRoles.length > 0 && roles.length > 0
              ? `${((realmRoles.length / roles.length) * 100).toFixed(0)}% of total`
              : 'no realm role',
        },
        {
          key: 'client',
          label: 'Client roles',
          value: clientRoles.length,
          hint: 'scoped to a client',
        },
        {
          key: 'granting',
          label: 'With permissions',
          value: withPermissions.length,
          hint: 'grant at least one',
        },
      ]}
      alerts={
        empty.length
          ? [
              {
                tone: 'warn' as const,
                title: `${empty.length} role grant${empty.length > 1 ? '' : 's'} nothing`,
                detail: `${empty.map((r) => r.name).join(', ')} — carries no permission.`,
                action: 'Review',
              },
            ]
          : []
      }
      filters={[
        { key: 'realm', label: 'Realm', predicate: (r) => !isClientRole(r) },
        { key: 'client', label: 'Client', predicate: isClientRole },
        { key: 'empty', label: 'Without permissions', predicate: (r) => r.permissions.length === 0 },
      ]}
      searchPlaceholder='Filter by name…'
      querySyntax='name:realm*  scope:client  permissions:0'
      searchIn={(r) => `${r.name} ${r.description ?? ''}`}
      rows={roles}
      columns={columns}
      card={card}
      getKey={(r) => r.id}
      getHref={roleHref}
      aggregates={{
        name: `${roles.length} role${roles.length !== 1 ? 's' : ''}`,
        permissions: roles.reduce((n, r) => n + r.permissions.length, 0),
      }}
      emptyLabel='No role'
      emptyHint='Roles group the permissions you grant to accounts and clients.'
      emptyAction={createButton}
    />
  )
}
