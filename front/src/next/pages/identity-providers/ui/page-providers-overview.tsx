import { KeyRound, Plus, Shield, Trash2 } from 'lucide-react'
import { Button } from '@/components/kit/button'
import { ConfirmDeleteAlert } from '@/components/confirm-delete-alert'
import { CreatePickerDialog, ListingPage, Pill, StatusDot } from '@/components/kit'
import type { CardSpec, Choice, Column, PillTone } from '@/components/kit'
import ProviderIcon from '@/pages/identity-providers/components/provider-icon'
import { PROVIDER_TEMPLATES } from '@/constants/identity-provider-templates'
import { Schemas } from '@/api/api.client'
import {
  providerName,
  providerStatus,
  providerTypeLabel,
  type ProviderProtocol,
} from '../provider-status'
import ProviderTile from './provider-tile'
import ProviderStatusPill from './provider-status-pill'

import IdentityProvider = Schemas.IdentityProviderResponse

interface ConfirmState {
  title: string
  description: string
  open: boolean
  onConfirm: () => void
}

export interface PageProvidersOverviewProps {
  providers: IdentityProvider[]
  isLoading: boolean
  pickerOpen: boolean
  confirm: ConfirmState
  providerHref: (provider: IdentityProvider) => string
  createUrl: (protocol: ProviderProtocol) => string
  onPickerOpenChange: (open: boolean) => void
  onQuickCreate: (templateId: string) => void
  onDelete: (provider: IdentityProvider) => void
  onConfirmClose: () => void
}

const POPULAR_TEMPLATE_IDS = ['google', 'discord', 'github', 'microsoft']

const popularTemplates = PROVIDER_TEMPLATES.filter((template) =>
  POPULAR_TEMPLATE_IDS.includes(template.id)
)

const typeTones: Record<string, PillTone> = {
  oidc: 'violet',
  oauth2: 'amber',
  saml: 'success',
  ldap: 'info',
}

const typeTone = (providerId: string) => typeTones[providerId.toLowerCase()] ?? 'primary'

const protocolChoices: Choice<ProviderProtocol>[] = [
  {
    value: 'oidc',
    label: 'OIDC',
    description: 'Identity claims come from the ID token, with a userinfo endpoint as a fallback.',
    icon: KeyRound,
  },
  {
    value: 'oauth2',
    label: 'OAuth2',
    description: 'Plain authorization code flow; the profile is read from a userinfo endpoint.',
    icon: KeyRound,
  },
  {
    value: 'saml',
    label: 'SAML',
    description: 'Federating a SAML identity provider is not implemented yet.',
    icon: Shield,
    disabledReason: 'FerrisKey cannot broker a SAML provider — the broker only speaks OAuth2 and OIDC.',
  },
  {
    value: 'ldap',
    label: 'LDAP',
    description: 'A directory is a user federation source, not an identity provider.',
    icon: Shield,
    disabledReason: 'A directory is configured under User Federation, not here.',
  },
]

export default function PageProvidersOverview({
  providers,
  isLoading,
  pickerOpen,
  confirm,
  providerHref,
  createUrl,
  onPickerOpenChange,
  onQuickCreate,
  onDelete,
  onConfirmClose,
}: PageProvidersOverviewProps) {
  const columns: Column<IdentityProvider>[] = [
    {
      key: 'provider',
      header: 'Provider',
      render: (p) => providerName(p),
      sortValue: (p) => providerName(p),
    },
    {
      key: 'alias',
      header: 'Alias',
      render: (p) => <span className='font-mono-ui text-xs text-neutral-500'>{p.alias}</span>,
      sortValue: (p) => p.alias,
    },
    {
      key: 'type',
      header: 'Type',
      render: (p) => (
        <Pill tone={typeTone(p.provider_id)} mono>
          {providerTypeLabel(p.provider_id)}
        </Pill>
      ),
      sortValue: (p) => p.provider_id,
    },
    {
      key: 'configuration',
      header: 'Configuration',
      render: (p) => {
        const status = providerStatus(p)
        return (
          <div className='min-w-0'>
            <ProviderStatusPill health={status.health} label={status.label} />
            <p className='mt-0.5 truncate text-xs text-neutral-500'>{status.detail}</p>
          </div>
        )
      },
      sortValue: (p) => providerStatus(p).health,
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
    {
      key: 'actions',
      header: '',
      align: 'right',
      render: (p) => (
        <Button
          variant='ghost'
          size='icon'
          aria-label={`Delete ${providerName(p)}`}
          className='text-neutral-400 hover:text-fk-danger'
          onClick={() => onDelete(p)}
        >
          <Trash2 className='size-4' />
        </Button>
      ),
    },
  ]

  const card: CardSpec<IdentityProvider> = {
    avatar: (p) => <ProviderTile providerId={p.provider_id} />,
    title: (p) => providerName(p),
    subtitle: (p) => p.alias,
    badges: (p) => {
      const status = providerStatus(p)
      return (
        <>
          <Pill tone={typeTone(p.provider_id)} mono>
            {providerTypeLabel(p.provider_id)}
          </Pill>
          <ProviderStatusPill health={status.health} label={status.label} />
          <Pill tone={p.enabled ? 'success' : 'neutral'}>
            <StatusDot on={p.enabled} />
            {p.enabled ? 'enabled' : 'disabled'}
          </Pill>
        </>
      )
    },
    flags: (p) => [
      { label: 'Offered on the login page', on: p.enabled },
      { label: 'Email address trusted as verified', on: p.trust_email },
      { label: 'Links to existing accounts only', on: p.link_only },
    ],
    footer: (p) => <span className='truncate'>{providerStatus(p).detail}</span>,
  }

  const enabled = providers.filter((p) => p.enabled)
  const disabled = providers.filter((p) => !p.enabled)
  const types = new Set(providers.map((p) => p.provider_id))
  const broken = providers.filter((p) => providerStatus(p).health === 'error')
  const degraded = providers.filter((p) => providerStatus(p).health === 'degraded')

  const addButton = (
    <Button onClick={() => onPickerOpenChange(true)}>
      <Plus /> Add provider
    </Button>
  )

  return (
    <>
      <ListingPage
        title='Identity Providers'
        description='External authentication sources federated into this realm.'
        loading={isLoading}
        actions={addButton}
        metrics={[
          {
            key: 'total',
            label: 'Total providers',
            value: providers.length,
            hint: 'configured',
          },
          {
            key: 'enabled',
            label: 'Enabled providers',
            value: enabled.length,
            hint:
              enabled.length > 0 && providers.length > 0
                ? `${((enabled.length / providers.length) * 100).toFixed(0)}% active`
                : 'no enabled provider',
          },
          {
            key: 'disabled',
            label: 'Disabled providers',
            value: disabled.length,
            hint: 'hidden from the login page',
          },
          {
            key: 'types',
            label: 'Provider types',
            value: types.size,
            hint: 'distinct protocols',
          },
        ]}
        alerts={[
          ...broken.map((p) => ({
            tone: 'error' as const,
            title: `${providerName(p)} cannot broker a login`,
            detail: providerStatus(p).detail,
            action: 'Fix',
          })),
          ...degraded.map((p) => ({
            tone: 'warn' as const,
            title: `${providerName(p)} requests no scope`,
            detail: providerStatus(p).detail,
            action: 'Review',
          })),
        ]}
        filters={[
          { key: 'enabled', label: 'Enabled', predicate: (p) => p.enabled },
          { key: 'disabled', label: 'Disabled', predicate: (p) => !p.enabled },
          {
            key: 'issues',
            label: 'Misconfigured',
            predicate: (p) => providerStatus(p).health !== 'healthy',
          },
        ]}
        searchPlaceholder='Filter by name or alias…'
        querySyntax='alias:git*  type:oidc  status:disabled'
        searchIn={(p) => `${p.display_name ?? ''} ${p.alias}`}
        rows={providers}
        columns={columns}
        card={card}
        getKey={(p) => p.alias}
        getHref={providerHref}
        aggregates={{
          provider: `${providers.length} provider${providers.length !== 1 ? 's' : ''}`,
          configuration: `${broken.length + degraded.length} to review`,
        }}
        emptyLabel='No identity provider'
        emptyHint='Federate an external provider so accounts can sign in with credentials they already have.'
        emptyAction={
          <div className='flex flex-col items-center gap-4'>
            {addButton}
            <div>
              <p className='text-center text-xs text-neutral-500'>Popular providers</p>
              <div className='mt-2 flex items-center justify-center gap-2'>
                {popularTemplates.map((template) => (
                  <button
                    key={template.id}
                    type='button'
                    onClick={() => onQuickCreate(template.id)}
                    className='flex cursor-pointer flex-col items-center gap-1.5 rounded-md px-2.5 py-2 transition-colors hover:bg-neutral-50'
                  >
                    <ProviderIcon icon={template.icon} size='sm' />
                    <span className='text-xs text-neutral-500'>{template.displayName}</span>
                  </button>
                ))}
              </div>
            </div>
          </div>
        }
      />

      <CreatePickerDialog
        open={pickerOpen}
        onOpenChange={onPickerOpenChange}
        title='Add an identity provider'
        description='The protocol decides which endpoints the provider must declare, so it is chosen before the form.'
        options={protocolChoices}
        defaultValue='oidc'
        createUrl={createUrl}
      />

      <ConfirmDeleteAlert
        title={confirm.title}
        description={confirm.description}
        open={confirm.open}
        onConfirm={confirm.onConfirm}
        onCancel={onConfirmClose}
      />
    </>
  )
}
