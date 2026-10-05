import { KeyRound, Plus, Shield, Trash2 } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/kit/button'
import { ConfirmDeleteAlert } from '@/components/confirm-delete-alert'
import { CreatePickerDialog, ListingPage, Pill, StatusDot } from '@/components/kit'
import type {
  CardSpec,
  Choice,
  Column,
  PagedListing,
  PaginationMetadata,
  PillTone,
} from '@/components/kit'
import ProviderIcon from '@/components/provider-icon'
import {
  ALL_TEMPLATES,
  PROVIDER_TEMPLATES,
  templateDisplayName,
} from '@/constants/identity-provider-templates'
import { Schemas } from '@/api/api.client'
import { formatRelative } from '@/utils/format-date'
import {
  PROVIDER_TYPES,
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

export interface ProviderCounts {
  total: number
  enabled: number
  disabled: number
}

export interface ProviderPreview {
  total: number
  names: string[]
}

const NAME_SEPARATOR = ', '
const TRUNCATION_MARK = '…'

const previewNames = (preview: ProviderPreview) =>
  preview.names.join(NAME_SEPARATOR) +
  (preview.total > preview.names.length ? TRUNCATION_MARK : '')

export interface PageProvidersOverviewProps {
  providers: IdentityProvider[]
  pagination: PaginationMetadata | undefined
  listing: PagedListing
  isLoading: boolean
  counts: ProviderCounts
  broken: ProviderPreview
  degraded: ProviderPreview
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

const HEALTH_ERROR = 'error'

const HEALTH_DEGRADED = 'degraded'

const YES = 'true'

const NO = 'false'

const popularTemplates = PROVIDER_TEMPLATES.filter((template) =>
  POPULAR_TEMPLATE_IDS.includes(template.id)
)

const providerTypeOptions = () => {
  const options = new Map<string, string>()
  PROVIDER_TYPES.forEach((type) => options.set(type, providerTypeLabel(type)))
  ALL_TEMPLATES.forEach((template) => {
    if (!options.has(template.name)) options.set(template.name, templateDisplayName(template))
  })
  return [...options].map(([value, label]) => ({ value, label }))
}

const typeTones: Record<string, PillTone> = {
  oidc: 'violet',
  oauth2: 'amber',
  saml: 'success',
  ldap: 'info',
}

const typeTone = (providerId: string) => typeTones[providerId.toLowerCase()] ?? 'primary'

export default function PageProvidersOverview({
  providers,
  pagination,
  listing,
  isLoading,
  counts,
  broken,
  degraded,
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
      sortKey: 'display_name',
      filters: [
        { kind: 'text', key: 'display_name', label: t('list.filter_fields.display_name') },
      ],
    },
    {
      key: 'alias',
      header: t('list.columns.alias'),
      render: (p) => <span className='font-mono-ui text-xs text-neutral-500 dark:text-neutral-400'>{p.alias}</span>,
      sortKey: 'alias',
      filters: [{ kind: 'text', key: 'alias', label: t('list.filter_fields.alias') }],
    },
    {
      key: 'type',
      header: t('list.columns.type'),
      render: (p) => (
        <Pill tone={typeTone(p.provider_id)} mono>
          {providerTypeLabel(p.provider_id)}
        </Pill>
      ),
      sortKey: 'provider_id',
      filters: [
        {
          kind: 'enum',
          key: 'provider_id',
          label: t('list.filter_fields.provider_id'),
          options: providerTypeOptions(),
        },
      ],
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
      filters: [
        {
          kind: 'enum',
          key: 'health',
          label: t('list.filter_fields.health'),
          options: [
            { value: 'healthy', label: t('list.filter_fields.health_values.healthy') },
            { value: 'degraded', label: t('list.filter_fields.health_values.degraded') },
            { value: 'error', label: t('list.filter_fields.health_values.error') },
          ],
        },
      ],
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
      sortKey: 'enabled',
      filters: [{ kind: 'boolean', key: 'enabled', label: t('list.filter_fields.enabled') }],
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
      filters: [
        {
          kind: 'date-range',
          fromKey: 'created_from',
          toKey: 'created_to',
          label: t('list.filter_fields.created'),
        },
      ],
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

  const reviewHealth = (health: string) => () =>
    listing.setFilters({ health })

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
            value: counts.total,
            hint: t('list.metrics.total.hint'),
            series: [counts.total, counts.total],
            filter: {},
          },
          {
            key: 'enabled',
            label: t('list.metrics.enabled.label'),
            value: counts.enabled,
            hint:
              counts.enabled > 0 && counts.total > 0
                ? t('list.metrics.enabled.hint', {
                    percent: ((counts.enabled / counts.total) * 100).toFixed(0),
                  })
                : t('list.metrics.enabled.empty_hint'),
            series: [counts.enabled, counts.enabled],
            filter: { enabled: YES },
          },
          {
            key: 'disabled',
            label: t('list.metrics.disabled.label'),
            value: counts.disabled,
            hint: t('list.metrics.disabled.hint'),
            series: [counts.disabled, counts.disabled],
            filter: { enabled: NO },
          },
        ]}
        alerts={[
          ...(broken.total
            ? [
                {
                  tone: 'error' as const,
                  title: t('list.alerts.broken.title', { count: broken.total }),
                  detail: t('list.alerts.broken.detail', {
                    count: broken.total,
                    names: previewNames(broken),
                  }),
                  action: t('list.alerts.broken.action'),
                  onAction: reviewHealth(HEALTH_ERROR),
                },
              ]
            : []),
          ...(degraded.total
            ? [
                {
                  tone: 'warn' as const,
                  title: t('list.alerts.degraded.title', { count: degraded.total }),
                  detail: t('list.alerts.degraded.detail', {
                    count: degraded.total,
                    names: previewNames(degraded),
                  }),
                  action: t('list.alerts.degraded.action'),
                  onAction: reviewHealth(HEALTH_DEGRADED),
                },
              ]
            : []),
        ]}
        paged={{ listing, pagination, search: { placeholder: t('list.search_placeholder') } }}
        rows={providers}
        columns={columns}
        card={card}
        getKey={(p) => p.alias}
        getHref={providerHref}
        aggregates={{
          provider: t('list.aggregates.provider_count', { count: counts.total }),
          configuration: t('list.aggregates.to_review', {
            total: broken.total + degraded.total,
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
