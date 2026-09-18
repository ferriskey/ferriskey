import { useMemo, useState } from 'react'
import { Building2, Search, UserMinus } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/kit/button'
import { EntityPicker, IconTile, Pill, Section } from '@/components/kit'
import { tokens } from '@/styles/style-tokens'
import { Schemas } from '@/api/api.client'

import Organization = Schemas.Organization
import { formatDate } from '@/utils/format-date'

export interface UserMembership {
  organization: Organization
  joinedAt: string
}

export interface UserOrganizationsTabProps {
  memberships: UserMembership[]
  availableOrganizations: Organization[]
  isLoading: boolean
  isError: boolean
  selectedOrganizationIds: string[]
  onSelectedOrganizationIdsChange: (next: string[]) => void
  onAssign: () => void
  onRemove: (organizationId: string) => void
}

const formatJoinedAt = (iso: string) =>
  formatDate(iso)

export default function UserOrganizationsTab({
  memberships,
  availableOrganizations,
  isLoading,
  isError,
  selectedOrganizationIds,
  onSelectedOrganizationIdsChange,
  onAssign,
  onRemove,
}: UserOrganizationsTabProps) {
  const { t } = useTranslation('user')
  const [query, setQuery] = useState('')

  const rows = useMemo(() => {
    const q = query.trim().toLowerCase()
    if (!q) return memberships
    return memberships.filter(({ organization }) =>
      `${organization.name} ${organization.alias} ${organization.domain ?? ''}`
        .toLowerCase()
        .includes(q)
    )
  }, [memberships, query])

  return (
    <>
      <Section
        title={t('detail.organizations.list.title')}
        description={t('detail.organizations.list.description')}
        action={
          memberships.length > 0 ? (
            <label className='relative flex h-7 w-48 items-center'>
              <Search className='pointer-events-none absolute left-2 size-3.5 text-neutral-400 dark:text-neutral-500' />
              <input
                type='search'
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                placeholder={t('detail.organizations.list.search_placeholder')}
                className='h-full w-full rounded-md border border-fk-line pl-7 pr-2 text-xs outline-none placeholder:text-neutral-400 focus:border-fk-primary-border'
              />
            </label>
          ) : undefined
        }
        contained={!isLoading && !isError && rows.length > 0}
      >
        {isError ? (
          <p className='rounded-md border border-dashed border-fk-danger-border bg-fk-danger-soft/40 px-4 py-3 text-sm text-neutral-600 dark:text-neutral-400'>
            {t('detail.organizations.list.error')}
          </p>
        ) : isLoading ? (
          <div className='space-y-2'>
            {Array.from({ length: 2 }).map((_, i) => (
              <div key={i} className='h-11 animate-pulse rounded-md bg-neutral-100 dark:bg-fk-raised' />
            ))}
          </div>
        ) : rows.length > 0 ? (
          <ul className={tokens.surface.divider}>
            {rows.map(({ organization, joinedAt }) => (
              <li key={organization.id} className='flex items-start gap-3 py-2.5'>
                <IconTile tone={organization.enabled ? 'info' : 'amber'}>
                  <Building2 className='size-4' strokeWidth={1.75} />
                </IconTile>

                <div className='min-w-0 flex-1'>
                  <div className='flex flex-wrap items-center gap-2'>
                    <p className='truncate text-sm font-medium text-neutral-900 dark:text-neutral-100'>
                      {organization.name}
                    </p>
                    <Pill mono>{organization.alias}</Pill>
                    {!organization.enabled && (
                      <Pill tone='amber'>{t('detail.organizations.list.disabled')}</Pill>
                    )}
                  </div>
                  <p className='mt-0.5 truncate text-xs text-neutral-500 dark:text-neutral-400'>
                    {t('detail.organizations.list.meta', {
                      date: formatJoinedAt(joinedAt),
                      detail:
                        organization.domain ??
                        t('detail.organizations.list.identifier', { id: organization.id }),
                    })}
                  </p>
                </div>

                <Button
                  variant='ghost'
                  size='icon'
                  aria-label={t('detail.organizations.list.remove', { name: organization.name })}
                  onClick={() => onRemove(organization.id)}
                  className='size-7 shrink-0 text-neutral-400 dark:text-neutral-500 hover:text-fk-danger'
                >
                  <UserMinus />
                </Button>
              </li>
            ))}
          </ul>
        ) : (
          <p className='rounded-md border border-dashed border-fk-line px-4 py-3 text-sm text-neutral-500 dark:text-neutral-400'>
            {query
              ? t('detail.organizations.list.no_match', { query })
              : t('detail.organizations.list.empty')}
          </p>
        )}
      </Section>

      <Section
        title={t('detail.organizations.assign.title')}
        description={t('detail.organizations.assign.description')}
        contained={false}
      >
        <div className='space-y-3'>
          <EntityPicker
            items={availableOrganizations.map((organization) => ({
              id: organization.id,
              label: organization.name,
              sublabel: organization.alias,
            }))}
            value={selectedOrganizationIds}
            onChange={onSelectedOrganizationIdsChange}
            addLabel={t('detail.organizations.assign.add')}
            searchPlaceholder={t('detail.organizations.assign.search_placeholder')}
            emptyHint={t('detail.organizations.assign.empty_hint')}
            exhaustedHint={t('detail.organizations.assign.exhausted_hint')}
          />
          <Button
            size='sm'
            disabled={selectedOrganizationIds.length === 0}
            onClick={onAssign}
          >
            {selectedOrganizationIds.length === 0
              ? t('detail.organizations.assign.submit_empty')
              : t('detail.organizations.assign.submit', {
                  count: selectedOrganizationIds.length,
                })}
          </Button>
        </div>
      </Section>
    </>
  )
}
