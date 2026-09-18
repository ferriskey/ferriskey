import { useMemo, useState } from 'react'
import { Search, Trash2 } from 'lucide-react'
import { useTranslation } from 'react-i18next'
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
  const { t } = useTranslation('user')
  const [query, setQuery] = useState('')

  const roleSubtitle = (role: Role) =>
    role.description ||
    (role.client?.client_id
      ? t('detail.role_mapping.role.client_ref', { clientId: role.client.client_id })
      : t('detail.role_mapping.role.identifier', { id: role.id }))

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
          {
            key: 'total',
            label: t('detail.role_mapping.metrics.total.label'),
            value: total,
            hint: t('detail.role_mapping.metrics.total.hint'),
          },
          {
            key: 'realm',
            label: t('detail.role_mapping.metrics.realm.label'),
            value: realmRoles,
            hint:
              realmRoles > 0 && total > 0
                ? t('detail.role_mapping.metrics.realm.hint', {
                    percent: ((realmRoles / total) * 100).toFixed(0),
                  })
                : t('detail.role_mapping.metrics.realm.empty_hint'),
          },
          {
            key: 'client',
            label: t('detail.role_mapping.metrics.client.label'),
            value: clientRoles,
            hint: t('detail.role_mapping.metrics.client.hint'),
          },
          {
            key: 'granting',
            label: t('detail.role_mapping.metrics.granting.label'),
            value: withPermissions,
            hint: t('detail.role_mapping.metrics.granting.hint'),
          },
        ]}
      />

      <Section
        title={t('detail.role_mapping.list.title')}
        description={t('detail.role_mapping.list.description')}
        action={
          roles.length > 0 ? (
            <label className='relative flex h-7 w-48 items-center'>
              <Search className='pointer-events-none absolute left-2 size-3.5 text-neutral-400 dark:text-neutral-500' />
              <input
                type='search'
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                placeholder={t('detail.role_mapping.list.search_placeholder')}
                className='h-full w-full rounded-md border border-fk-line pl-7 pr-2 text-xs outline-none placeholder:text-neutral-400 focus:border-fk-primary-border'
              />
            </label>
          ) : undefined
        }
        contained={!isLoading && !isError && rows.length > 0}
      >
        {isError ? (
          <p className='rounded-md border border-dashed border-fk-danger-border bg-fk-danger-soft/40 px-4 py-3 text-sm text-neutral-600 dark:text-neutral-400'>
            {t('detail.role_mapping.list.error')}
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
                      {role.client_id
                        ? t('detail.role_mapping.role.client')
                        : t('detail.role_mapping.role.realm')}
                    </Pill>
                    <Pill tone={role.permissions.length > 0 ? 'success' : 'amber'}>
                      {t('detail.role_mapping.role.permission_count', {
                        count: role.permissions.length,
                      })}
                    </Pill>
                  </div>
                  <p className='mt-0.5 truncate text-xs text-neutral-500 dark:text-neutral-400'>
                    {roleSubtitle(role)}
                  </p>
                </div>
                <Button
                  variant='ghost'
                  size='icon'
                  aria-label={t('detail.role_mapping.list.unassign', { name: role.name })}
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
              ? t('detail.role_mapping.list.no_match', { query })
              : t('detail.role_mapping.list.empty')}
          </p>
        )}
      </Section>

      <Section
        title={t('detail.role_mapping.assign.title')}
        description={t('detail.role_mapping.assign.description')}
        contained={false}
      >
        <div className='space-y-3'>
          <EntityPicker
            items={assignableRoles.map((role) => ({
              id: role.id,
              label: role.name,
              sublabel: role.client?.client_id
                ? t('detail.role_mapping.role.client_ref', { clientId: role.client.client_id })
                : t('detail.role_mapping.role.realm'),
            }))}
            value={selectedRoleIds}
            onChange={onSelectedRoleIdsChange}
            addLabel={t('detail.role_mapping.assign.add')}
            searchPlaceholder={t('detail.role_mapping.assign.search_placeholder')}
            emptyHint={t('detail.role_mapping.assign.empty_hint')}
            exhaustedHint={t('detail.role_mapping.assign.exhausted_hint')}
          />
          <Button size='sm' disabled={selectedRoleIds.length === 0} onClick={onAssign}>
            {selectedRoleIds.length === 0
              ? t('detail.role_mapping.assign.submit_empty')
              : t('detail.role_mapping.assign.submit', { count: selectedRoleIds.length })}
          </Button>
        </div>
      </Section>
    </>
  )
}
