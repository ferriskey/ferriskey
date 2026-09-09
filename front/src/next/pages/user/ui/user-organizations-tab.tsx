import { useMemo, useState } from 'react'
import { Building2, Search, UserMinus } from 'lucide-react'
import { Button } from '@/components/kit/button'
import { EntityPicker, IconTile, MetricsBand, Pill, Section } from '@/components/kit'
import { tokens } from '@/styles/style-tokens'
import { Schemas } from '@/api/api.client'

import Organization = Schemas.Organization
import { formatDate } from '@/next/shared/format-date'

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

  const total = memberships.length
  const enabled = memberships.filter((m) => m.organization.enabled).length
  const withDomain = memberships.filter((m) => Boolean(m.organization.domain)).length

  return (
    <>
      <MetricsBand
        metrics={[
          { key: 'total', label: 'Organizations', value: total, hint: 'memberships' },
          {
            key: 'enabled',
            label: 'Enabled',
            value: enabled,
            hint:
              enabled > 0 && total > 0
                ? `${((enabled / total) * 100).toFixed(0)}% of total`
                : 'No enabled organizations',
          },
          {
            key: 'disabled',
            label: 'Disabled',
            value: total - enabled,
            hint: 'stop granting anything',
          },
          { key: 'domain', label: 'With domain', value: withDomain, hint: 'domain configured' },
        ]}
      />

      <Section
        title='Organizations'
        description='Organizations of the realm this account belongs to.'
        action={
          memberships.length > 0 ? (
            <label className='relative flex h-7 w-48 items-center'>
              <Search className='pointer-events-none absolute left-2 size-3.5 text-neutral-400 dark:text-neutral-500' />
              <input
                type='search'
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                placeholder='Filter organizations…'
                className='h-full w-full rounded-md border border-fk-line pl-7 pr-2 text-xs outline-none placeholder:text-neutral-400 focus:border-fk-primary-border'
              />
            </label>
          ) : undefined
        }
        contained={!isLoading && !isError && rows.length > 0}
      >
        {isError ? (
          <p className='rounded-md border border-dashed border-fk-danger-border bg-fk-danger-soft/40 px-4 py-3 text-sm text-neutral-600 dark:text-neutral-400'>
            The organizations of this account could not be loaded.
          </p>
        ) : isLoading ? (
          <div className='space-y-2'>
            {Array.from({ length: 2 }).map((_, i) => (
              <div key={i} className='h-11 animate-pulse rounded-md bg-neutral-100 dark:bg-neutral-800' />
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
                    {!organization.enabled && <Pill tone='amber'>organization disabled</Pill>}
                  </div>
                  <p className='mt-0.5 truncate text-xs text-neutral-500 dark:text-neutral-400'>
                    Member since {formatJoinedAt(joinedAt)} ·{' '}
                    {organization.domain ?? `org_id: ${organization.id}`}
                  </p>
                </div>

                <Button
                  variant='ghost'
                  size='icon'
                  aria-label={`Remove from ${organization.name}`}
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
              ? `No organization matches “${query}”.`
              : 'This account belongs to no organization.'}
          </p>
        )}
      </Section>

      <Section
        title='Add to an organization'
        description='Organizations of the realm this account has not joined yet.'
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
            addLabel='Pick an organization'
            searchPlaceholder='Search an organization…'
            emptyHint='No organization selected yet.'
            exhaustedHint='Every available organization is already selected.'
          />
          <Button
            size='sm'
            disabled={selectedOrganizationIds.length === 0}
            onClick={onAssign}
          >
            Add to {selectedOrganizationIds.length > 0 ? selectedOrganizationIds.length : ''}{' '}
            organization{selectedOrganizationIds.length > 1 ? 's' : ''}
          </Button>
        </div>
      </Section>
    </>
  )
}
