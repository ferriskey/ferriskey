import { useMemo, useState } from 'react'
import { Search } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Section, IconTile } from '@/components/kit'
import {
  permissionCatalogue,
  permissionCount,
  permissionDescriptionKey,
  permissionGroupLabelKey,
  permissionLabelKey,
} from '../permission-catalogue'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'

export interface RolePermissionsTabProps {
  value: string[]
  onChange: (next: string[]) => void
}

export default function RolePermissionsTab({ value, onChange }: RolePermissionsTabProps) {
  const { t } = useTranslation('role')
  const [query, setQuery] = useState('')

  const groups = useMemo(() => {
    const q = query.trim().toLowerCase()
    return permissionCatalogue
      .map((g) => ({
        key: g.key,
        icon: g.icon,
        total: g.permissions.length,
        permissions: q
          ? g.permissions.filter((p) =>
              `${p} ${t(permissionLabelKey(p))} ${t(permissionDescriptionKey(p))}`
                .toLowerCase()
                .includes(q)
            )
          : [...g.permissions],
      }))
      .filter((g) => g.permissions.length > 0)
  }, [query, t])

  const toggle = (key: string) =>
    onChange(value.includes(key) ? value.filter((x) => x !== key) : [...value, key])

  const selectAll = (keys: string[]) => onChange([...new Set([...value, ...keys])])
  const clearAll = (keys: string[]) => onChange(value.filter((v) => !keys.includes(v)))

  return (
    <Section
      title={t('permissions.title')}
      description={t('permissions.description')}
      contained={false}
    >
      <div className='space-y-3'>
        <div className='flex flex-wrap items-center gap-3'>
          <label className='relative flex h-8 min-w-0 flex-1 items-center sm:max-w-sm'>
            <Search className='pointer-events-none absolute left-2.5 size-3.5 text-neutral-400 dark:text-neutral-500' />
            <input
              type='search'
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder={t('permissions.search_placeholder')}
              className='h-full w-full rounded-md border border-fk-line pl-8 pr-2 text-xs outline-none placeholder:text-neutral-400 focus:border-fk-primary-border'
            />
          </label>
          <span className='tnum text-xs text-neutral-500 dark:text-neutral-400'>
            {t('permissions.granted', { granted: value.length, total: permissionCount })}
          </span>
          {value.length > 0 && (
            <button
              type='button'
              onClick={() => onChange([])}
              className='cursor-pointer text-xs text-neutral-500 dark:text-neutral-400 underline-offset-2 hover:text-fk-danger hover:underline'
            >
              {t('permissions.clear_all')}
            </button>
          )}
        </div>

        {groups.length === 0 ? (
          <p className='rounded-lg border border-dashed border-fk-line px-4 py-3 text-xs text-neutral-500 dark:text-neutral-400'>
            {t('permissions.no_match', { query })}
          </p>
        ) : (
          <div className='grid gap-3 lg:grid-cols-2 xl:grid-cols-3'>
            {groups.map((group) => {
              const keys = [...group.permissions]
              const enabled = keys.filter((k) => value.includes(k)).length
              const all = enabled === keys.length

              return (
                <section key={group.key} className={cn(tokens.surface.panel, 'flex flex-col p-4')}>
                  <div className='flex items-start gap-2.5'>
                    <IconTile tone={enabled > 0 ? 'primary' : 'info'}>
                      <group.icon className='size-4' strokeWidth={1.75} />
                    </IconTile>
                    <div className='min-w-0 flex-1'>
                      <p className='text-sm font-semibold text-neutral-900 dark:text-neutral-100'>
                        {t(permissionGroupLabelKey(group.key))}
                      </p>
                      <p
                        className={cn(
                          'tnum text-xs',
                          enabled > 0 ? 'text-fk-primary-text' : 'text-neutral-500 dark:text-neutral-400'
                        )}
                      >
                        {t('permissions.group_enabled', { enabled, total: group.total })}
                      </p>
                    </div>
                    <button
                      type='button'
                      onClick={() => (all ? clearAll(keys) : selectAll(keys))}
                      className='shrink-0 cursor-pointer text-xs text-neutral-400 dark:text-neutral-500 underline-offset-2 hover:text-fk-primary-text hover:underline'
                    >
                      {all ? t('permissions.select_none') : t('permissions.select_all')}
                    </button>
                  </div>

                  <div className='mt-3 space-y-1.5'>
                    {group.permissions.map((p) => {
                      const on = value.includes(p)
                      return (
                        <button
                          key={p}
                          type='button'
                          role='switch'
                          aria-checked={on}
                          title={t(permissionDescriptionKey(p))}
                          onClick={() => toggle(p)}
                          className={cn(
                            'flex w-full cursor-pointer items-center gap-2 rounded-md border px-2.5 py-2 text-left transition-colors',
                            'focus-visible:outline-none focus-visible:ring-3 focus-visible:ring-fk-primary/30',
                            on
                              ? 'border-fk-primary-border bg-fk-primary-soft'
                              : 'border-fk-line bg-white hover:bg-neutral-50 dark:bg-fk-surface dark:hover:bg-fk-surface'
                          )}
                        >
                          <span className='min-w-0 flex-1'>
                            <span
                              className={cn(
                                'block truncate text-xs',
                                on ? 'font-medium text-fk-primary-text' : 'text-neutral-700 dark:text-neutral-300'
                              )}
                            >
                              {t(permissionLabelKey(p))}
                            </span>
                            <span className='block truncate font-mono-ui text-[11px] text-neutral-400 dark:text-neutral-500'>
                              {p}
                            </span>
                          </span>
                          <span
                            className={cn(
                              'shrink-0 rounded border px-1.5 py-0.5 text-[11px] leading-5',
                              on
                                ? 'border-fk-primary-border bg-white dark:bg-fk-surface text-fk-primary-text'
                                : 'border-fk-line bg-neutral-50 text-neutral-400 dark:bg-fk-surface dark:text-neutral-500'
                            )}
                          >
                            {on ? t('permissions.state.enabled') : t('permissions.state.disabled')}
                          </span>
                        </button>
                      )
                    })}
                  </div>
                </section>
              )
            })}
          </div>
        )}
      </div>
    </Section>
  )
}
