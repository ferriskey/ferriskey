import { Database, KeyRound, Plus, Server } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/kit/button'
import {
  CreatePickerDialog,
  IconTile,
  ListingPage,
  Pill,
  StatusDot,
  type CardSpec,
  type Choice,
  type Column,
  type FilterField,
  type PagedListing,
  type PaginationMetadata,
} from '@/components/kit'
import { Schemas } from '@/api/api.client'
import { formatRelative } from '@/utils/format-date'
import {
  LDAP_PROVIDER_TYPE,
  PRIORITY_LABEL_KEY,
  USER_FEDERATION_NAMESPACE,
  asLdapConfig,
  formatSyncedAt,
  isLdapLike,
  ldapConnectionUrl,
  priorityFromScore,
} from '../provider-config'

import ProviderResponse = Schemas.ProviderResponse

export type FederationKind = 'Ldap' | 'Kerberos'

export interface ProviderCounts {
  total: number
  enabled: number
  disabled: number
  scheduled: number
}

export interface ProviderPreview {
  total: number
  names: string[]
}

export interface PageProvidersOverviewProps {
  providers: ProviderResponse[]
  pagination: PaginationMetadata | undefined
  listing: PagedListing
  isLoading: boolean
  counts: ProviderCounts
  neverSynced: ProviderPreview
  pickerOpen: boolean
  onPickerOpenChange: (open: boolean) => void
  createUrl: (kind: FederationKind) => string
  providerHref: (provider: ProviderResponse) => string
}

const KERBEROS_PROVIDER_TYPE = 'Kerberos'

const ACTIVE_DIRECTORY_PROVIDER_TYPE = 'ActiveDirectory'

const PROVIDER_TYPE_OPTIONS = [
  LDAP_PROVIDER_TYPE,
  ACTIVE_DIRECTORY_PROVIDER_TYPE,
  KERBEROS_PROVIDER_TYPE,
].map((value) => ({ value, label: value }))

const NAME_SEPARATOR = ', '
const TRUNCATION_MARK = '…'

const previewNames = (preview: ProviderPreview) =>
  preview.names.join(NAME_SEPARATOR) +
  (preview.total > preview.names.length ? TRUNCATION_MARK : '')

const KIND_CHOICES = [
  { value: LDAP_PROVIDER_TYPE, labelKey: 'kind.ldap', icon: Database, disabled: false },
  { value: KERBEROS_PROVIDER_TYPE, labelKey: 'kind.kerberos', icon: KeyRound, disabled: true },
] as const

const endpointOf = (provider: ProviderResponse) =>
  isLdapLike(provider.provider_type) ? ldapConnectionUrl(asLdapConfig(provider.config)) : ''

const tileTone = (provider: ProviderResponse) =>
  provider.provider_type === LDAP_PROVIDER_TYPE ? 'violet' : ('info' as const)

const kindIcon = (provider: ProviderResponse) =>
  isLdapLike(provider.provider_type) ? (
    <Database className='size-4' strokeWidth={1.75} />
  ) : provider.provider_type === KERBEROS_PROVIDER_TYPE ? (
    <KeyRound className='size-4' strokeWidth={1.75} />
  ) : (
    <Server className='size-4' strokeWidth={1.75} />
  )

export default function PageProvidersOverview({
  providers,
  pagination,
  listing,
  isLoading,
  counts,
  neverSynced,
  pickerOpen,
  onPickerOpenChange,
  createUrl,
  providerHref,
}: PageProvidersOverviewProps) {
  const { t } = useTranslation(USER_FEDERATION_NAMESPACE)

  const statusLabel = (enabled: boolean) =>
    enabled ? t('list.status.enabled') : t('list.status.disabled')

  const cardStatusLabel = (enabled: boolean) =>
    enabled ? t('list.card.status.enabled') : t('list.card.status.disabled')

  const kindChoices: Choice<FederationKind>[] = KIND_CHOICES.map((choice) => ({
    value: choice.value,
    label: t(`${choice.labelKey}.label`),
    description: t(`${choice.labelKey}.description`),
    icon: choice.icon,
    disabledReason: choice.disabled ? t(`${choice.labelKey}.disabled_reason`) : undefined,
  }))

  const columns: Column<ProviderResponse>[] = [
    {
      key: 'name',
      header: t('list.columns.name'),
      render: (p) => p.name,
      sortKey: 'name',
    },
    {
      key: 'endpoint',
      header: t('list.columns.endpoint'),
      render: (p) => {
        const endpoint = endpointOf(p)
        return endpoint ? (
          <span className='font-mono-ui text-xs text-neutral-600 dark:text-neutral-400'>{endpoint}</span>
        ) : (
          <span className='font-mono-ui text-xs text-neutral-400 dark:text-neutral-500'>
            {t('list.no_endpoint')}
          </span>
        )
      },
    },
    {
      key: 'provider_type',
      header: t('list.columns.type'),
      render: (p) => (
        <Pill tone={p.provider_type === LDAP_PROVIDER_TYPE ? 'violet' : 'info'} mono>
          {p.provider_type}
        </Pill>
      ),
    },
    {
      key: 'priority',
      header: t('list.columns.priority'),
      render: (p) => (
        <span className='text-neutral-600 dark:text-neutral-400'>
          {t(PRIORITY_LABEL_KEY[priorityFromScore(p.priority)])}
        </span>
      ),
      sortKey: 'priority',
    },
    {
      key: 'sync_mode',
      header: t('list.columns.sync_mode'),
      render: (p) => <Pill mono>{p.sync_mode}</Pill>,
    },
    {
      key: 'schedule',
      header: t('list.columns.schedule'),
      render: (p) =>
        p.sync_enabled && p.sync_interval_minutes ? (
          <span className='tnum text-neutral-600 dark:text-neutral-400'>
            {t('list.schedule.interval', { count: p.sync_interval_minutes })}
          </span>
        ) : (
          <span className='text-neutral-400 dark:text-neutral-500'>
            {t('list.schedule.on_demand')}
          </span>
        ),
    },
    {
      key: 'last_sync',
      header: t('list.columns.last_sync'),
      render: (p) =>
        formatSyncedAt(p.last_sync_at) ?? (
          <span className='text-neutral-400 dark:text-neutral-500'>
            {t('list.never_synced')}
          </span>
        ),
      sortKey: 'last_sync_at',
    },
    {
      key: 'status',
      header: t('list.columns.status'),
      render: (p) => (
        <span className='inline-flex items-center gap-1.5 text-xs text-neutral-600 dark:text-neutral-400'>
          <StatusDot on={p.enabled} />
          {statusLabel(p.enabled)}
        </span>
      ),
      sortKey: 'enabled',
    },
    {
      key: 'created',
      header: t('list.columns.created'),
      render: (p) => (
        <span className='tnum text-neutral-600 dark:text-neutral-400'>
          {formatRelative(p.created_at)}
        </span>
      ),
      sortKey: 'created_at',
    },
    {
      key: 'updated',
      header: t('list.columns.updated'),
      render: (p) => (
        <span className='tnum text-neutral-600 dark:text-neutral-400'>
          {formatRelative(p.updated_at)}
        </span>
      ),
      sortKey: 'updated_at',
    },
  ]

  const card: CardSpec<ProviderResponse> = {
    avatar: (p) => <IconTile tone={tileTone(p)}>{kindIcon(p)}</IconTile>,
    title: (p) => p.name,
    subtitle: (p) => endpointOf(p) || t('list.no_endpoint'),
    badges: (p) => (
      <>
        <Pill tone={p.provider_type === LDAP_PROVIDER_TYPE ? 'violet' : 'info'} mono>
          {p.provider_type}
        </Pill>
        <Pill mono>{p.sync_mode}</Pill>
        <Pill tone={p.enabled ? 'success' : 'neutral'}>
          <StatusDot on={p.enabled} />
          {cardStatusLabel(p.enabled)}
        </Pill>
      </>
    ),
    flags: (p) => [
      { label: t('list.card.flags.queried'), on: p.enabled },
      { label: t('list.card.flags.scheduled'), on: p.sync_enabled },
      { label: t('list.card.flags.synced'), on: Boolean(p.last_sync_at) },
    ],
    footer: (p) => (
      <>
        <span>{t(PRIORITY_LABEL_KEY[priorityFromScore(p.priority)])}</span>
        <span>{formatSyncedAt(p.last_sync_at) ?? t('list.never_synced')}</span>
      </>
    ),
  }

  const filterFields: FilterField[] = [
    { kind: 'text', key: 'name', label: t('list.filter_fields.name') },
    {
      kind: 'enum',
      key: 'provider_type',
      label: t('list.filter_fields.provider_type'),
      options: PROVIDER_TYPE_OPTIONS,
    },
    { kind: 'boolean', key: 'enabled', label: t('list.filter_fields.enabled') },
    { kind: 'boolean', key: 'sync_enabled', label: t('list.filter_fields.sync_enabled') },
    { kind: 'boolean', key: 'synced', label: t('list.filter_fields.synced') },
  ]

  const reviewNeverSynced = () => listing.setFilters({ synced: 'false' })

  const createButton = (
    <Button onClick={() => onPickerOpenChange(true)}>
      <Plus /> {t('list.create')}
    </Button>
  )

  return (
    <>
      <ListingPage
        title={t('list.title')}
        description={t('list.description')}
        loading={isLoading}
        actions={createButton}
        metrics={[
          {
            key: 'total',
            label: t('list.metrics.total.label'),
            value: counts.total,
            hint: t('list.metrics.total.hint'),
            series: [counts.total, counts.total],
          },
          {
            key: 'enabled',
            label: t('list.metrics.enabled.label'),
            value: counts.enabled,
            hint:
              counts.disabled > 0
                ? t('list.metrics.enabled.hint', { total: counts.disabled })
                : t('list.metrics.enabled.empty_hint'),
            series: [counts.enabled, counts.enabled],
          },
          {
            key: 'scheduled',
            label: t('list.metrics.scheduled.label'),
            value: counts.scheduled,
            hint: t('list.metrics.scheduled.hint'),
            series: [counts.scheduled, counts.scheduled],
          },
          {
            key: 'stale',
            label: t('list.metrics.stale.label'),
            value: neverSynced.total,
            hint:
              neverSynced.total > 0
                ? t('list.metrics.stale.hint')
                : t('list.metrics.stale.empty_hint'),
            series: [neverSynced.total, neverSynced.total],
          },
        ]}
        alerts={
          neverSynced.total
            ? [
                {
                  tone: 'warn' as const,
                  title: t('list.alerts.stale.title', { count: neverSynced.total }),
                  detail: t('list.alerts.stale.detail', {
                    names: previewNames(neverSynced),
                  }),
                  action: t('list.alerts.stale.action'),
                  onAction: reviewNeverSynced,
                },
              ]
            : []
        }
        paged={{ listing, pagination, filterFields }}
        rows={providers}
        columns={columns}
        card={card}
        getKey={(p) => p.id}
        getHref={providerHref}
        aggregates={{
          name: t('list.count', { count: counts.total }),
          status: t('list.aggregates.status', { total: counts.enabled }),
        }}
        emptyLabel={t('list.empty.label')}
        emptyHint={t('list.empty.hint')}
        emptyAction={createButton}
      />

      <CreatePickerDialog
        open={pickerOpen}
        onOpenChange={onPickerOpenChange}
        title={t('list.picker.title')}
        description={t('list.picker.description')}
        options={kindChoices}
        defaultValue={LDAP_PROVIDER_TYPE}
        createUrl={createUrl}
      />
    </>
  )
}
