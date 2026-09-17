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
} from '@/components/kit'
import { Schemas } from '@/api/api.client'
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

export interface PageProvidersOverviewProps {
  providers: ProviderResponse[]
  isLoading: boolean
  pickerOpen: boolean
  onPickerOpenChange: (open: boolean) => void
  createUrl: (kind: FederationKind) => string
  providerHref: (provider: ProviderResponse) => string
}

const QUERY_SYNTAX = 'type:Ldap  enabled:true  synced:never'

const KERBEROS_PROVIDER_TYPE = 'Kerberos'

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
  isLoading,
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
      sortValue: (p) => p.name,
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
      sortValue: (p) => p.provider_type,
    },
    {
      key: 'sync_mode',
      header: t('list.columns.sync_mode'),
      render: (p) => <Pill mono>{p.sync_mode}</Pill>,
      sortValue: (p) => p.sync_mode,
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
      sortValue: (p) => (p.sync_enabled ? (p.sync_interval_minutes ?? 0) : 0),
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
      sortValue: (p) => p.last_sync_at ?? '',
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
      sortValue: (p) => statusLabel(p.enabled),
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

  const enabled = providers.filter((p) => p.enabled)
  const scheduled = providers.filter((p) => p.sync_enabled)
  const neverSynced = providers.filter((p) => !p.last_sync_at)

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
            value: providers.length,
            hint: t('list.metrics.total.hint'),
          },
          {
            key: 'enabled',
            label: t('list.metrics.enabled.label'),
            value: enabled.length,
            hint:
              providers.length - enabled.length > 0
                ? t('list.metrics.enabled.hint', { total: providers.length - enabled.length })
                : t('list.metrics.enabled.empty_hint'),
          },
          {
            key: 'scheduled',
            label: t('list.metrics.scheduled.label'),
            value: scheduled.length,
            hint: t('list.metrics.scheduled.hint'),
          },
          {
            key: 'stale',
            label: t('list.metrics.stale.label'),
            value: neverSynced.length,
            hint:
              neverSynced.length > 0
                ? t('list.metrics.stale.hint')
                : t('list.metrics.stale.empty_hint'),
          },
        ]}
        alerts={
          neverSynced.length
            ? [
                {
                  tone: 'warn' as const,
                  title: t('list.alerts.stale.title', { count: neverSynced.length }),
                  detail: t('list.alerts.stale.detail', {
                    names: neverSynced.map((p) => p.name).join(', '),
                  }),
                },
              ]
            : []
        }
        filters={[
          { key: 'enabled', label: t('list.filters.enabled'), predicate: (p) => p.enabled },
          {
            key: 'ldap',
            label: t('list.filters.ldap'),
            predicate: (p) => isLdapLike(p.provider_type),
          },
          {
            key: 'scheduled',
            label: t('list.filters.scheduled'),
            predicate: (p) => p.sync_enabled,
          },
          { key: 'stale', label: t('list.filters.stale'), predicate: (p) => !p.last_sync_at },
        ]}
        searchPlaceholder={t('list.search_placeholder')}
        querySyntax={QUERY_SYNTAX}
        searchIn={(p) => `${p.name} ${p.provider_type} ${endpointOf(p)}`}
        rows={providers}
        columns={columns}
        card={card}
        getKey={(p) => p.id}
        getHref={providerHref}
        aggregates={{
          name: t('list.count', { count: providers.length }),
          status: t('list.aggregates.status', { total: enabled.length }),
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
