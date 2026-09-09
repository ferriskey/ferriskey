import { useMemo, useState } from 'react'
import { Search, Shield, Trash2 } from 'lucide-react'
import { Button } from '@/components/kit/button'
import { ConfirmDeleteAlert } from '@/components/confirm-delete-alert'
import { useConfirmDeleteAlert } from '@/hooks/use-confirm-delete-alert'
import { IconTile, Section } from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import { Schemas } from '@/api/api.client'

import Role = Schemas.Role

export interface ClientRolesTabProps {
  roles: Role[]
  isLoading: boolean
  isError: boolean
  onDeleteRole: (role: Role) => void
}

export default function ClientRolesTab({
  roles,
  isLoading,
  isError,
  onDeleteRole,
}: ClientRolesTabProps) {
  const { confirm, ask, close } = useConfirmDeleteAlert()
  const [query, setQuery] = useState('')

  const rows = useMemo(() => {
    const q = query.trim().toLowerCase()
    if (!q) return roles
    return roles.filter((r) => `${r.name} ${r.description ?? ''}`.toLowerCase().includes(q))
  }, [roles, query])

  if (isError) {
    return (
      <Section
        title='Client roles'
        description='Roles defined on this client, exposed in tokens under resource_access.'
        contained={false}
      >
        <p className='rounded-lg border border-dashed border-fk-danger-border bg-fk-danger-soft/40 px-4 py-3 text-xs text-fk-danger'>
          Error while loading roles.
        </p>
      </Section>
    )
  }

  const askDelete = (role: Role) =>
    ask({
      title: 'Delete role?',
      description: `Are you sure you want to delete "${role.name}"? This action cannot be undone.`,
      onConfirm: () => {
        onDeleteRole(role)
        close()
      },
    })

  return (
    <>
      <Section
        title={`Client roles (${roles.length})`}
        description='Roles defined on this client, exposed in tokens under resource_access.'
        action={
          <label className='relative flex h-8 w-52 items-center'>
            <Search className='pointer-events-none absolute left-2.5 size-3.5 text-neutral-400' />
            <input
              type='search'
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder='Filter roles…'
              className='h-full w-full rounded-md border border-fk-line pl-8 pr-2 text-xs outline-none placeholder:text-neutral-400 focus:border-fk-primary-border'
            />
          </label>
        }
        contained={rows.length > 0}
      >
        {isLoading ? (
          <div className={cn(tokens.surface.panel, tokens.surface.divider)}>
            {Array.from({ length: 3 }).map((_, i) => (
              <div key={i} className='flex items-center gap-3 px-3 py-3'>
                <div className='size-7 animate-pulse rounded-md bg-neutral-100' />
                <div className='h-3 w-40 animate-pulse rounded bg-neutral-100' />
              </div>
            ))}
          </div>
        ) : rows.length > 0 ? (
          <ul className={tokens.surface.divider}>
            {rows.map((role) => (
              <li key={role.id} className='flex items-center gap-3 py-2.5'>
                <IconTile tone='info' className='size-7'>
                  <Shield className='size-3.5' strokeWidth={1.75} />
                </IconTile>

                <div className='min-w-0 flex-1'>
                  <p className='font-mono-ui text-xs text-neutral-900'>{role.name}</p>
                  <p className='mt-0.5 truncate text-xs text-neutral-500'>
                    {role.description || 'No description'}
                  </p>
                </div>

                <span className='tnum shrink-0 text-xs text-neutral-400'>
                  {role.permissions.length} permission{role.permissions.length === 1 ? '' : 's'}
                </span>

                <Button
                  variant='ghost'
                  size='icon'
                  aria-label={`Delete ${role.name}`}
                  onClick={() => askDelete(role)}
                  className='size-7 text-neutral-400 hover:text-fk-danger'
                >
                  <Trash2 />
                </Button>
              </li>
            ))}
          </ul>
        ) : (
          <p className='rounded-lg border border-dashed border-fk-line px-4 py-3 text-xs text-neutral-500'>
            {query
              ? `No role matches “${query}”.`
              : 'No role of its own — holders of this client only get realm roles.'}
          </p>
        )}
      </Section>

      <ConfirmDeleteAlert
        title={confirm.title}
        description={confirm.description}
        open={confirm.open}
        onConfirm={confirm.onConfirm}
        onCancel={close}
      />
    </>
  )
}
