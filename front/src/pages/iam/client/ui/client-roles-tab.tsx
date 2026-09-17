import { useMemo, useState } from 'react'
import { Search, Shield, Trash2 } from 'lucide-react'
import { useTranslation } from 'react-i18next'
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
  const { t } = useTranslation('client')
  const { confirm, ask, close } = useConfirmDeleteAlert()
  const [query, setQuery] = useState('')

  const rows = useMemo(() => {
    const q = query.trim().toLowerCase()
    if (!q) return roles
    return roles.filter((r) => `${r.name} ${r.description ?? ''}`.toLowerCase().includes(q))
  }, [roles, query])

  if (isError) {
    return (
      <Section title={t('roles.title')} description={t('roles.description')} contained={false}>
        <p className='rounded-lg border border-dashed border-fk-danger-border bg-fk-danger-soft/40 px-4 py-3 text-xs text-fk-danger'>
          {t('roles.error')}
        </p>
      </Section>
    )
  }

  const askDelete = (role: Role) =>
    ask({
      title: t('roles.delete.title'),
      description: t('roles.delete.description', { name: role.name }),
      onConfirm: () => {
        onDeleteRole(role)
        close()
      },
    })

  return (
    <>
      <Section
        title={t('roles.title_with_count', { total: roles.length })}
        description={t('roles.description')}
        action={
          <label className='relative flex h-8 w-52 items-center'>
            <Search className='pointer-events-none absolute left-2.5 size-3.5 text-neutral-400 dark:text-neutral-500' />
            <input
              type='search'
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder={t('roles.search_placeholder')}
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
                <div className='size-7 animate-pulse rounded-md bg-neutral-100 dark:bg-fk-raised' />
                <div className='h-3 w-40 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
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
                  <p className='font-mono-ui text-xs text-neutral-900 dark:text-neutral-100'>{role.name}</p>
                  <p className='mt-0.5 truncate text-xs text-neutral-500 dark:text-neutral-400'>
                    {role.description || t('shared.no_description')}
                  </p>
                </div>

                <span className='tnum shrink-0 text-xs text-neutral-400 dark:text-neutral-500'>
                  {t('roles.permissions', { count: role.permissions.length })}
                </span>

                <Button
                  variant='ghost'
                  size='icon'
                  aria-label={t('roles.delete.label', { name: role.name })}
                  onClick={() => askDelete(role)}
                  className='size-7 text-neutral-400 dark:text-neutral-500 hover:text-fk-danger'
                >
                  <Trash2 />
                </Button>
              </li>
            ))}
          </ul>
        ) : (
          <p className='rounded-lg border border-dashed border-fk-line px-4 py-3 text-xs text-neutral-500 dark:text-neutral-400'>
            {query ? t('roles.empty.no_match', { query }) : t('roles.empty.none')}
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
