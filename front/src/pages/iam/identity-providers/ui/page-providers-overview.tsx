import { KeyRound, Plus, Shield, Trash2 } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/kit/button'
import { ConfirmDeleteAlert } from '@/components/confirm-delete-alert'
import { CreatePickerDialog, ListingPage, Pill, StatusDot } from '@/components/kit'
import type { CardSpec, Choice, Column, PillTone } from '@/components/kit'
import ProviderIcon from '@/components/provider-icon'
import {
  PROVIDER_TEMPLATES,
  templateDisplayName,
} from '@/constants/identity-provider-templates'
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

const DEFAULT_PROTOCOL: ProviderProtocol = 'oidc'

const QUERY_SYNTAX_HINT = 'alias:git*  type:oidc  status:disabled'

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
  const { t } = useTranslation('identity-provider')

  const protocolChoices: Choice<ProviderProtocol>[] = [
    {
      value: 'oidc',
      label: t('provider_type.oidc'),
      description: t('list.picker.oidc.description'),
      icon: KeyRound,
    },
    {
      value: 'oauth2',
      label: t('provider_type.oauth2'),
      description: t('list.picker.oauth2.description'),
      icon: KeyRound,
    },
    {
      value: 'saml',
      label: t('provider_type.saml'),
      description: t('list.picker.saml.description'),
      icon: Shield,
      disabledReason: t('list.picker.saml.disabled_reason'),
    },
    {
      value: 'ldap',
      label: t('provider_type.ldap'),
      description: t('list.picker.ldap.description'),
      icon: Shield,
      disabledReason: t('list.picker.ldap.disabled_reason'),
    },
  ]

  const columns: Column<IdentityProvider>[] = [
    {
      key: 'provider',
      header: t('list.columns.provider'),
      render: (p) => providerName(p),
      sortValue: (p) => providerName(p),
    },
    {
      key: 'alias',
      header: t('list.columns.alias'),
      render: (p) => <span className='font-mono-ui text-xs text-neutral-500 dark:text-neutral-400'>{p.alias}</span>,
      sortValue: (p) => p.alias,
    },
    {
      key: 'type',
      header: t('list.columns.type'),
      render: (p) => (
        <Pill tone={typeTone(p.provider_id)} mono>
          {providerTypeLabel(p.provider_id)}
        </Pill>
      ),
      sortValue: (p) => p.provider_id,
    },
    {
      key: 'configuration',
      header: t('list.columns.configuration'),
      render: (p) => {
        const status = providerStatus(p)
        return (
          <div className='min-w-0'>
            <ProviderStatusPill health={status.health} label={status.label} />
            <p className='mt-0.5 truncate text-xs text-neutral-500 dark:text-neutral-400'>{status.detail}</p>
          </div>
        )
      },
      sortValue: (p) => providerStatus(p).health,
    },
    {
      key: 'status',
      header: t('list.columns.status'),
      render: (p) => (
        <span className='inline-flex items-center gap-1.5 text-xs text-neutral-600 dark:text-neutral-400'>
          <StatusDot on={p.enabled} />
          {p.enabled ? t('state.enabled') : t('state.disabled')}
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
          aria-label={t('list.row_delete', { name: providerName(p) })}
          className='text-neutral-400 dark:text-neutral-500 hover:text-fk-danger'
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
            {p.enabled ? t('badge.enabled') : t('badge.disabled')}
          </Pill>
        </>
      )
    },
    flags: (p) => [
      { label: t('list.flags.enabled'), on: p.enabled },
      { label: t('list.flags.trust_email'), on: p.trust_email },
      { label: t('list.flags.link_only'), on: p.link_only },
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
      <Plus /> {t('list.add')}
    </Button>
  )

  return (
    <>
      <ListingPage
        title={t('list.title')}
        description={t('list.description')}
        loading={isLoading}
        actions={addButton}
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
              enabled.length > 0 && providers.length > 0
                ? t('list.metrics.enabled.hint', {
                    percent: ((enabled.length / providers.length) * 100).toFixed(0),
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
            key: 'types',
            label: t('list.metrics.types.label'),
            value: types.size,
            hint: t('list.metrics.types.hint'),
          },
        ]}
        alerts={[
          ...broken.map((p) => ({
            tone: 'error' as const,
            title: t('list.alerts.broken.title', { name: providerName(p) }),
            detail: providerStatus(p).detail,
            action: t('list.alerts.broken.action'),
          })),
          ...degraded.map((p) => ({
            tone: 'warn' as const,
            title: t('list.alerts.degraded.title', { name: providerName(p) }),
            detail: providerStatus(p).detail,
            action: t('list.alerts.degraded.action'),
          })),
        ]}
        filters={[
          { key: 'enabled', label: t('list.filters.enabled'), predicate: (p) => p.enabled },
          { key: 'disabled', label: t('list.filters.disabled'), predicate: (p) => !p.enabled },
          {
            key: 'issues',
            label: t('list.filters.issues'),
            predicate: (p) => providerStatus(p).health !== 'healthy',
          },
        ]}
        searchPlaceholder={t('list.search_placeholder')}
        querySyntax={QUERY_SYNTAX_HINT}
        searchIn={(p) => `${p.display_name ?? ''} ${p.alias}`}
        rows={providers}
        columns={columns}
        card={card}
        getKey={(p) => p.alias}
        getHref={providerHref}
        aggregates={{
          provider: t('list.aggregates.provider_count', { count: providers.length }),
          configuration: t('list.aggregates.to_review', {
            total: broken.length + degraded.length,
          }),
        }}
        emptyLabel={t('list.empty.label')}
        emptyHint={t('list.empty.hint')}
        emptyAction={
          <div className='flex flex-col items-center gap-4'>
            {addButton}
            <div>
              <p className='text-center text-xs text-neutral-500 dark:text-neutral-400'>
                {t('list.empty.popular')}
              </p>
              <div className='mt-2 flex items-center justify-center gap-2'>
                {popularTemplates.map((template) => (
                  <button
                    key={template.id}
                    type='button'
                    onClick={() => onQuickCreate(template.id)}
                    className='flex cursor-pointer flex-col items-center gap-1.5 rounded-md px-2.5 py-2 transition-colors hover:bg-neutral-50 dark:hover:bg-fk-surface'
                  >
                    <ProviderIcon icon={template.icon} size='sm' />
                    <span className='text-xs text-neutral-500 dark:text-neutral-400'>
                      {templateDisplayName(template)}
                    </span>
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
        title={t('list.picker.title')}
        description={t('list.picker.description')}
        options={protocolChoices}
        defaultValue={DEFAULT_PROTOCOL}
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
