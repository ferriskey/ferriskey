import { Database, KeyRound, Plus, Server } from 'lucide-react'
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
  asLdapConfig,
  formatDateTime,
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

const kindChoices: Choice<FederationKind>[] = [
  {
    value: 'Ldap',
    label: 'LDAP',
    description: 'A directory queried to authenticate accounts and import them.',
    icon: Database,
  },
  {
    value: 'Kerberos',
    label: 'Kerberos',
    description: 'Ticket-based network authentication.',
    icon: KeyRound,
    disabledReason:
      'The server stores a Kerberos provider but neither synchronises nor tests it.',
  },
]

const endpointOf = (provider: ProviderResponse) =>
  isLdapLike(provider.provider_type) ? ldapConnectionUrl(asLdapConfig(provider.config)) : ''

const tileTone = (provider: ProviderResponse) =>
  provider.provider_type === 'Ldap' ? 'violet' : ('info' as const)

const kindIcon = (provider: ProviderResponse) =>
  isLdapLike(provider.provider_type) ? (
    <Database className='size-4' strokeWidth={1.75} />
  ) : provider.provider_type === 'Kerberos' ? (
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
  const columns: Column<ProviderResponse>[] = [
    {
      key: 'name',
      header: 'Provider',
      render: (p) => p.name,
      sortValue: (p) => p.name,
    },
    {
      key: 'endpoint',
      header: 'Endpoint',
      render: (p) => {
        const endpoint = endpointOf(p)
        return endpoint ? (
          <span className='font-mono-ui text-xs text-neutral-600'>{endpoint}</span>
        ) : (
          <span className='font-mono-ui text-xs text-neutral-400'>no endpoint</span>
        )
      },
    },
    {
      key: 'provider_type',
      header: 'Type',
      render: (p) => (
        <Pill tone={p.provider_type === 'Ldap' ? 'violet' : 'info'} mono>
          {p.provider_type}
        </Pill>
      ),
      sortValue: (p) => p.provider_type,
    },
    {
      key: 'sync_mode',
      header: 'Sync mode',
      render: (p) => <Pill mono>{p.sync_mode}</Pill>,
      sortValue: (p) => p.sync_mode,
    },
    {
      key: 'schedule',
      header: 'Schedule',
      render: (p) =>
        p.sync_enabled && p.sync_interval_minutes ? (
          <span className='tnum text-neutral-600'>every {p.sync_interval_minutes} min</span>
        ) : (
          <span className='text-neutral-400'>on demand only</span>
        ),
      sortValue: (p) => (p.sync_enabled ? (p.sync_interval_minutes ?? 0) : 0),
    },
    {
      key: 'last_sync',
      header: 'Last sync',
      render: (p) =>
        formatDateTime(p.last_sync_at) ?? <span className='text-neutral-400'>never synced</span>,
      sortValue: (p) => p.last_sync_at ?? '',
    },
    {
      key: 'status',
      header: 'Status',
      render: (p) => (
        <span className='inline-flex items-center gap-1.5 text-xs text-neutral-600'>
          <StatusDot on={p.enabled} />
          {p.enabled ? 'Enabled' : 'Disabled'}
        </span>
      ),
      sortValue: (p) => (p.enabled ? 'enabled' : 'disabled'),
    },
  ]

  const card: CardSpec<ProviderResponse> = {
    avatar: (p) => <IconTile tone={tileTone(p)}>{kindIcon(p)}</IconTile>,
    title: (p) => p.name,
    subtitle: (p) => endpointOf(p) || 'no endpoint',
    badges: (p) => (
      <>
        <Pill tone={p.provider_type === 'Ldap' ? 'violet' : 'info'} mono>
          {p.provider_type}
        </Pill>
        <Pill mono>{p.sync_mode}</Pill>
        <Pill tone={p.enabled ? 'success' : 'neutral'}>
          <StatusDot on={p.enabled} />
          {p.enabled ? 'enabled' : 'disabled'}
        </Pill>
      </>
    ),
    flags: (p) => [
      { label: 'Queried for authentication', on: p.enabled },
      { label: 'Scheduled synchronisation', on: p.sync_enabled },
      { label: 'Already synchronised', on: Boolean(p.last_sync_at) },
    ],
    footer: (p) => (
      <>
        <span>{priorityFromScore(p.priority)}</span>
        <span>{formatDateTime(p.last_sync_at) ?? 'never synced'}</span>
      </>
    ),
  }

  const enabled = providers.filter((p) => p.enabled)
  const scheduled = providers.filter((p) => p.sync_enabled)
  const neverSynced = providers.filter((p) => !p.last_sync_at)

  const createButton = (
    <Button onClick={() => onPickerOpenChange(true)}>
      <Plus /> Add provider
    </Button>
  )

  return (
    <>
      <ListingPage
        title='User Federation'
        description='External directories whose accounts are imported into this realm.'
        loading={isLoading}
        actions={createButton}
        metrics={[
          { key: 'total', label: 'Providers', value: providers.length, hint: 'configured' },
          {
            key: 'enabled',
            label: 'Enabled',
            value: enabled.length,
            hint:
              providers.length - enabled.length > 0
                ? `${providers.length - enabled.length} disabled`
                : 'all queried',
          },
          {
            key: 'scheduled',
            label: 'Scheduled sync',
            value: scheduled.length,
            hint: 'run without being asked',
          },
          {
            key: 'stale',
            label: 'Never synced',
            value: neverSynced.length,
            hint: neverSynced.length > 0 ? 'no account imported' : 'all have run once',
          },
        ]}
        alerts={
          neverSynced.length
            ? [
                {
                  tone: 'warn' as const,
                  title: `${neverSynced.length} provider${neverSynced.length > 1 ? 's have' : ' has'} never synchronised`,
                  detail: `${neverSynced.map((p) => p.name).join(', ')} — no account imported so far.`,
                },
              ]
            : []
        }
        filters={[
          { key: 'enabled', label: 'Enabled', predicate: (p) => p.enabled },
          { key: 'ldap', label: 'LDAP', predicate: (p) => isLdapLike(p.provider_type) },
          { key: 'scheduled', label: 'Scheduled', predicate: (p) => p.sync_enabled },
          { key: 'stale', label: 'Never synced', predicate: (p) => !p.last_sync_at },
        ]}
        searchPlaceholder='Filter by name or endpoint…'
        querySyntax='type:Ldap  enabled:true  synced:never'
        searchIn={(p) => `${p.name} ${p.provider_type} ${endpointOf(p)}`}
        rows={providers}
        columns={columns}
        card={card}
        getKey={(p) => p.id}
        getHref={providerHref}
        aggregates={{
          name: `${providers.length} provider${providers.length !== 1 ? 's' : ''}`,
          status: `${enabled.length} enabled`,
        }}
        emptyLabel='No federation provider'
        emptyHint='Connect a directory to authenticate its accounts and import them into this realm.'
        emptyAction={createButton}
      />

      <CreatePickerDialog
        open={pickerOpen}
        onOpenChange={onPickerOpenChange}
        title='Provider type'
        description='It settles the configuration to fill in: an LDAP directory and a Kerberos domain share no setting.'
        options={kindChoices}
        defaultValue='Ldap'
        createUrl={createUrl}
      />
    </>
  )
}
