import { useMemo, useState } from 'react'
import { Search, Trash2 } from 'lucide-react'
import { Button } from '@/components/kit/button'
import { EntityPicker, MetricsBand, Pill, Section } from '@/components/kit'
import { tokens } from '@/styles/style-tokens'
import { Schemas } from '@/api/api.client'

import Role = Schemas.Role

export interface UserRoleMappingTabProps {
  roles: Role[]
  assignableRoles: Role[]
  isLoading: boolean
  isError: boolean
  selectedRoleIds: string[]
  onSelectedRoleIdsChange: (next: string[]) => void
  onAssign: () => void
  onUnassign: (roleId: string) => void
}

const roleSubtitle = (role: Role) =>
  role.description ||
  (role.client?.client_id ? `client: ${role.client.client_id}` : `role_id: ${role.id}`)

export default function UserRoleMappingTab({
  roles,
  assignableRoles,
  isLoading,
  isError,
  selectedRoleIds,
  onSelectedRoleIdsChange,
  onAssign,
  onUnassign,
}: UserRoleMappingTabProps) {
  const [query, setQuery] = useState('')

  const rows = useMemo(() => {
    const q = query.trim().toLowerCase()
    if (!q) return roles
    return roles.filter((r) => `${r.name} ${r.description ?? ''}`.toLowerCase().includes(q))
  }, [roles, query])

  const total = roles.length
  const realmRoles = roles.filter((r) => !r.client_id).length
  const clientRoles = roles.filter((r) => Boolean(r.client_id)).length
  const withPermissions = roles.filter((r) => r.permissions.length > 0).length

  return (
    <>
      <MetricsBand
        metrics={[
          { key: 'total', label: 'Assigned roles', value: total, hint: 'on this account' },
          {
            key: 'realm',
            label: 'Realm roles',
            value: realmRoles,
            hint:
              realmRoles > 0 && total > 0
                ? `${((realmRoles / total) * 100).toFixed(0)}% of total`
                : 'No realm roles',
          },
          {
            key: 'client',
            label: 'Client roles',
            value: clientRoles,
            hint: 'scoped to a client',
          },
          {
            key: 'granting',
            label: 'With permissions',
            value: withPermissions,
            hint: 'grant at least one',
          },
        ]}
      />

      <Section
        title='Assigned roles'
        description='Everything this account is authorised to do comes from this list.'
        action={
          roles.length > 0 ? (
            <label className='relative flex h-7 w-48 items-center'>
              <Search className='pointer-events-none absolute left-2 size-3.5 text-neutral-400 dark:text-neutral-500' />
              <input
                type='search'
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                placeholder='Filter roles…'
                className='h-full w-full rounded-md border border-fk-line pl-7 pr-2 text-xs outline-none placeholder:text-neutral-400 focus:border-fk-primary-border'
              />
            </label>
          ) : undefined
        }
        contained={!isLoading && !isError && rows.length > 0}
      >
        {isError ? (
          <p className='rounded-md border border-dashed border-fk-danger-border bg-fk-danger-soft/40 px-4 py-3 text-sm text-neutral-600 dark:text-neutral-400'>
            The roles of this account could not be loaded.
          </p>
        ) : isLoading ? (
          <div className='space-y-2'>
            {Array.from({ length: 3 }).map((_, i) => (
              <div key={i} className='h-9 animate-pulse rounded-md bg-neutral-100 dark:bg-fk-raised' />
            ))}
          </div>
        ) : rows.length > 0 ? (
          <ul className={tokens.surface.divider}>
            {rows.map((role) => (
              <li key={role.id} className='flex items-center gap-3 py-2.5'>
                <div className='min-w-0 flex-1'>
                  <div className='flex flex-wrap items-center gap-2'>
                    <p className='font-mono-ui text-xs text-neutral-900 dark:text-neutral-100'>{role.name}</p>
                    <Pill tone={role.client_id ? 'violet' : 'info'} mono>
                      {role.client_id ? 'client' : 'realm'}
                    </Pill>
                    <Pill tone={role.permissions.length > 0 ? 'success' : 'amber'}>
                      {role.permissions.length} permission
                      {role.permissions.length !== 1 ? 's' : ''}
                    </Pill>
                  </div>
                  <p className='mt-0.5 truncate text-xs text-neutral-500 dark:text-neutral-400'>
                    {roleSubtitle(role)}
                  </p>
                </div>
                <Button
                  variant='ghost'
                  size='icon'
                  aria-label={`Unassign ${role.name}`}
                  onClick={() => onUnassign(role.id)}
                  className='size-7 shrink-0 text-neutral-400 dark:text-neutral-500 hover:text-fk-danger'
                >
                  <Trash2 />
                </Button>
              </li>
            ))}
          </ul>
        ) : (
          <p className='rounded-md border border-dashed border-fk-amber-border bg-fk-amber-soft/40 px-4 py-3 text-sm text-neutral-600 dark:text-neutral-400'>
            {query
              ? `No role matches “${query}”.`
              : 'No role assigned — this account cannot reach any protected resource.'}
          </p>
        )}
      </Section>

      <Section
        title='Assign a role'
        description='Roles of the realm and of its clients that this account does not hold yet.'
        contained={false}
      >
        <div className='space-y-3'>
          <EntityPicker
            items={assignableRoles.map((role) => ({
              id: role.id,
              label: role.name,
              sublabel: role.client?.client_id ? `client: ${role.client.client_id}` : 'realm',
            }))}
            value={selectedRoleIds}
            onChange={onSelectedRoleIdsChange}
            addLabel='Pick a role'
            searchPlaceholder='Search a role…'
            emptyHint='No role selected yet.'
            exhaustedHint='This account already holds every role of the realm.'
          />
          <Button size='sm' disabled={selectedRoleIds.length === 0} onClick={onAssign}>
            Assign {selectedRoleIds.length > 0 ? selectedRoleIds.length : ''} role
            {selectedRoleIds.length > 1 ? 's' : ''}
          </Button>
        </div>
      </Section>
    </>
  )
}
