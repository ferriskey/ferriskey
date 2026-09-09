import { Building2, Plus } from 'lucide-react'
import { Button } from '@/components/kit/button'
import { IconTile, ListingPage, Pill, StatusDot } from '@/components/kit'
import type { CardSpec, Column } from '@/components/kit'
import { Schemas } from '@/api/api.client'

import Organization = Schemas.Organization

export interface PageOrganizationsOverviewProps {
  organizations: Organization[]
  isLoading: boolean
  organizationHref: (organization: Organization) => string
  onCreate: () => void
  onReviewDisabled: () => void
}

export default function PageOrganizationsOverview({
  organizations,
  isLoading,
  organizationHref,
  onCreate,
  onReviewDisabled,
}: PageOrganizationsOverviewProps) {
  const columns: Column<Organization>[] = [
    {
      key: 'name',
      header: 'Organization',
      render: (o) => o.name,
      sortValue: (o) => o.name,
    },
    {
      key: 'alias',
      header: 'Alias',
      render: (o) => <span className='font-mono-ui text-xs text-neutral-500 dark:text-neutral-400'>{o.alias}</span>,
      sortValue: (o) => o.alias,
    },
    {
      key: 'domain',
      header: 'Domain',
      render: (o) =>
        o.domain ? (
          <span className='font-mono-ui text-xs text-neutral-600 dark:text-neutral-400'>{o.domain}</span>
        ) : (
          <span className='text-xs text-neutral-400 dark:text-neutral-500'>no domain</span>
        ),
      sortValue: (o) => o.domain ?? '',
    },
    {
      key: 'description',
      header: 'Description',
      render: (o) =>
        o.description ? (
          <span className='text-neutral-600 dark:text-neutral-400'>{o.description}</span>
        ) : (
          <span className='font-mono-ui text-xs text-neutral-400 dark:text-neutral-500'>
            organization_id: {o.id}
          </span>
        ),
    },
    {
      key: 'status',
      header: 'Status',
      render: (o) => (
        <span className='inline-flex items-center gap-1.5 text-xs text-neutral-600 dark:text-neutral-400'>
          <StatusDot on={o.enabled} />
          {o.enabled ? 'enabled' : 'disabled'}
        </span>
      ),
      sortValue: (o) => (o.enabled ? 'enabled' : 'disabled'),
    },
  ]

  const card: CardSpec<Organization> = {
    avatar: () => (
      <IconTile tone='violet'>
        <Building2 className='size-4' strokeWidth={1.75} />
      </IconTile>
    ),
    title: (o) => o.name,
    subtitle: (o) => o.alias,
    badges: (o) => (
      <>
        <Pill tone={o.enabled ? 'success' : 'neutral'}>
          <StatusDot on={o.enabled} />
          {o.enabled ? 'enabled' : 'disabled'}
        </Pill>
        {o.domain && <Pill mono>{o.domain}</Pill>}
      </>
    ),
    flags: (o) => [
      { label: 'Reachable by its members', on: o.enabled },
      { label: 'Bound to an email domain', on: Boolean(o.domain) },
      { label: 'Redirects after login', on: Boolean(o.redirect_url) },
    ],
    footer: (o) => (
      <span className='truncate'>{o.description || `organization_id: ${o.id}`}</span>
    ),
  }

  const enabled = organizations.filter((o) => o.enabled)
  const disabled = organizations.filter((o) => !o.enabled)
  const withDomain = organizations.filter((o) => Boolean(o.domain))

  const createButton = (
    <Button onClick={onCreate}>
      <Plus /> New organization
    </Button>
  )

  return (
    <ListingPage
      title='Organizations'
      description='Groups of accounts sharing a domain, a redirection and their own roles.'
      loading={isLoading}
      actions={createButton}
      metrics={[
        {
          key: 'total',
          label: 'Total',
          value: organizations.length,
          hint: `organization${organizations.length !== 1 ? 's' : ''}`,
        },
        {
          key: 'enabled',
          label: 'Enabled',
          value: enabled.length,
          hint:
            enabled.length > 0 && organizations.length > 0
              ? `${((enabled.length / organizations.length) * 100).toFixed(0)}% of total`
              : 'none enabled',
        },
        {
          key: 'disabled',
          label: 'Disabled',
          value: disabled.length,
          hint: 'hidden from end-user flows',
        },
        {
          key: 'domain',
          label: 'With a domain',
          value: withDomain.length,
          hint: 'bound to an email domain',
        },
      ]}
      alerts={
        disabled.length
          ? [
              {
                tone: 'warn' as const,
                title: `${disabled.length} organization${disabled.length > 1 ? 's are' : ' is'} disabled`,
                detail: `${disabled.map((o) => o.name).join(', ')} — members cannot reach ${disabled.length > 1 ? 'them' : 'it'}.`,
                action: 'Review',
                onAction: onReviewDisabled,
              },
            ]
          : []
      }
      filters={[
        { key: 'enabled', label: 'Enabled', predicate: (o) => o.enabled },
        { key: 'disabled', label: 'Disabled', predicate: (o) => !o.enabled },
        { key: 'nodomain', label: 'Without domain', predicate: (o) => !o.domain },
      ]}
      searchPlaceholder='Filter by name, alias or domain…'
      querySyntax='name:acme*  alias:acme  domain:*.com'
      searchIn={(o) => `${o.name} ${o.alias} ${o.domain ?? ''}`}
      rows={organizations}
      columns={columns}
      card={card}
      getKey={(o) => o.id}
      getHref={organizationHref}
      aggregates={{
        name: `${organizations.length} organization${organizations.length !== 1 ? 's' : ''}`,
        status: `${enabled.length} enabled`,
      }}
      emptyLabel='No organization'
      emptyHint='An organization groups the accounts of a same company, with its own domain and roles.'
      emptyAction={createButton}
    />
  )
}
