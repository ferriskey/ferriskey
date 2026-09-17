import { Building2, Plus } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/kit/button'
import { IconTile, ListingPage, Pill, StatusDot } from '@/components/kit'
import type { CardSpec, Column } from '@/components/kit'
import { Schemas } from '@/api/api.client'

import Organization = Schemas.Organization

const QUERY_SYNTAX = 'name:acme*  alias:acme  domain:*.com'

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
  const { t } = useTranslation('organization')

  const statusLabel = (enabled: boolean) =>
    enabled ? t('organization.status.enabled') : t('organization.status.disabled')

  const columns: Column<Organization>[] = [
    {
      key: 'name',
      header: t('list.columns.name'),
      render: (o) => o.name,
      sortValue: (o) => o.name,
    },
    {
      key: 'alias',
      header: t('list.columns.alias'),
      render: (o) => <span className='font-mono-ui text-xs text-neutral-500 dark:text-neutral-400'>{o.alias}</span>,
      sortValue: (o) => o.alias,
    },
    {
      key: 'domain',
      header: t('list.columns.domain'),
      render: (o) =>
        o.domain ? (
          <span className='font-mono-ui text-xs text-neutral-600 dark:text-neutral-400'>{o.domain}</span>
        ) : (
          <span className='text-xs text-neutral-400 dark:text-neutral-500'>
            {t('organization.no_domain')}
          </span>
        ),
      sortValue: (o) => o.domain ?? '',
    },
    {
      key: 'description',
      header: t('list.columns.description'),
      render: (o) =>
        o.description ? (
          <span className='text-neutral-600 dark:text-neutral-400'>{o.description}</span>
        ) : (
          <span className='font-mono-ui text-xs text-neutral-400 dark:text-neutral-500'>
            {t('organization.identifier', { id: o.id })}
          </span>
        ),
    },
    {
      key: 'status',
      header: t('list.columns.status'),
      render: (o) => (
        <span className='inline-flex items-center gap-1.5 text-xs text-neutral-600 dark:text-neutral-400'>
          <StatusDot on={o.enabled} />
          {statusLabel(o.enabled)}
        </span>
      ),
      sortValue: (o) => statusLabel(o.enabled),
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
          {statusLabel(o.enabled)}
        </Pill>
        {o.domain && <Pill mono>{o.domain}</Pill>}
      </>
    ),
    flags: (o) => [
      { label: t('list.card.flags.reachable'), on: o.enabled },
      { label: t('list.card.flags.domain'), on: Boolean(o.domain) },
      { label: t('list.card.flags.redirect'), on: Boolean(o.redirect_url) },
    ],
    footer: (o) => (
      <span className='truncate'>
        {o.description || t('organization.identifier', { id: o.id })}
      </span>
    ),
  }

  const enabled = organizations.filter((o) => o.enabled)
  const disabled = organizations.filter((o) => !o.enabled)
  const withDomain = organizations.filter((o) => Boolean(o.domain))

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
          value: organizations.length,
          hint: t('list.metrics.total.hint', { count: organizations.length }),
        },
        {
          key: 'enabled',
          label: t('list.metrics.enabled.label'),
          value: enabled.length,
          hint:
            enabled.length > 0 && organizations.length > 0
              ? t('list.metrics.enabled.hint', {
                  percent: ((enabled.length / organizations.length) * 100).toFixed(0),
                })
              : t('list.metrics.enabled.empty_hint'),
        },
        {
          key: 'disabled',
          label: t('list.metrics.disabled.label'),
          value: disabled.length,
          hint: t('list.metrics.disabled.hint'),
        },
        {
          key: 'domain',
          label: t('list.metrics.domain.label'),
          value: withDomain.length,
          hint: t('list.metrics.domain.hint'),
        },
      ]}
      alerts={
        disabled.length
          ? [
              {
                tone: 'warn' as const,
                title: t('list.alerts.disabled.title', { count: disabled.length }),
                detail: t('list.alerts.disabled.detail', {
                  count: disabled.length,
                  names: disabled.map((o) => o.name).join(', '),
                }),
                action: t('list.alerts.disabled.action'),
                onAction: onReviewDisabled,
              },
            ]
          : []
      }
      filters={[
        { key: 'enabled', label: t('list.filters.enabled'), predicate: (o) => o.enabled },
        { key: 'disabled', label: t('list.filters.disabled'), predicate: (o) => !o.enabled },
        { key: 'nodomain', label: t('list.filters.without_domain'), predicate: (o) => !o.domain },
      ]}
      searchPlaceholder={t('list.search_placeholder')}
      querySyntax={QUERY_SYNTAX}
      searchIn={(o) => `${o.name} ${o.alias} ${o.domain ?? ''}`}
      rows={organizations}
      columns={columns}
      card={card}
      getKey={(o) => o.id}
      getHref={organizationHref}
      aggregates={{
        name: t('list.count', { count: organizations.length }),
        status: t('list.aggregates.status', { total: enabled.length }),
      }}
      emptyLabel={t('list.empty.label')}
      emptyHint={t('list.empty.hint')}
      emptyAction={createButton}
    />
  )
}
