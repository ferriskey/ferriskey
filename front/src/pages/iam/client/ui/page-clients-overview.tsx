import { Plus } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/kit/button'
import { CreatePickerDialog, ListingPage, Pill, Squircle, StatusDot } from '@/components/kit'
import type {
  CardSpec,
  Column,
  PagedListing,
  PaginationMetadata,
} from '@/components/kit'
import { Schemas } from '@/api/api.client'
import { formatRelative } from '@/utils/format-date'
import {
  DEFAULT_PROTOCOL,
  clientAuthenticationOf,
  clientStateOf,
  protocolChoices,
  type ClientProtocol,
} from '../client-choices'

import Client = Schemas.Client

export interface ClientCounts {
  total: number
  active: number
  public: number
  confidential: number
}

export interface ClientPreview {
  total: number
  names: string[]
}

export interface PageClientsOverviewProps {
  clients: Client[]
  pagination: PaginationMetadata | undefined
  listing: PagedListing
  isLoading: boolean
  counts: ClientCounts
  withoutRedirect: ClientPreview
  inMaintenance: ClientPreview
  pickerOpen: boolean
  onPickerOpenChange: (open: boolean) => void
  createUrl: (protocol: ClientProtocol) => string
  clientHref: (client: Client) => string
}

const NAME_SEPARATOR = ', '
const TRUNCATION_MARK = '…'
const YES = 'true'
const NO = 'false'

const previewNames = (preview: ClientPreview) =>
  preview.names.join(NAME_SEPARATOR) +
  (preview.total > preview.names.length ? TRUNCATION_MARK : '')

const redirectCount = (client: Client) => client.redirect_uris?.length ?? 0

export default function PageClientsOverview({
  clients,
  pagination,
  listing,
  isLoading,
  counts,
  withoutRedirect,
  inMaintenance,
  pickerOpen,
  onPickerOpenChange,
  createUrl,
  clientHref,
}: PageClientsOverviewProps) {
  const { t } = useTranslation('client')

  const columns: Column<Client>[] = [
    {
      key: 'name',
      header: t('list.columns.name'),
      render: (c) => c.name,
      sortKey: 'name',
      filters: [{ kind: 'text', key: 'name', label: t('list.filter_fields.name') }],
    },
    {
      key: 'client_id',
      header: t('list.columns.client_id'),
      render: (c) => <span className='font-mono-ui text-xs text-neutral-500 dark:text-neutral-400'>{c.client_id}</span>,
      sortKey: 'client_id',
    },
    {
      key: 'kind',
      header: t('list.columns.kind'),
      render: (c) => (
        <Pill tone={c.public_client ? 'info' : 'violet'} mono>
          {t(`shared.authentication.${clientAuthenticationOf(c.public_client)}`)}
        </Pill>
      ),
      filters: [
        {
          kind: 'enum',
          key: 'client_type',
          label: t('list.filter_fields.client_type'),
          options: [
            { value: 'confidential', label: t('list.filter_fields.client_types.confidential') },
            { value: 'public', label: t('list.filter_fields.client_types.public') },
            { value: 'system', label: t('list.filter_fields.client_types.system') },
          ],
        },
        { kind: 'boolean', key: 'public_client', label: t('list.filter_fields.public_client') },
        {
          kind: 'boolean',
          key: 'service_account_enabled',
          label: t('list.filter_fields.service_account_enabled'),
        },
      ],
    },
    {
      key: 'protocol',
      header: t('list.columns.protocol'),
      render: (c) => (
        <Pill tone='primary' mono>
          {c.protocol}
        </Pill>
      ),
      filters: [
        {
          kind: 'enum',
          key: 'protocol',
          label: t('list.filter_fields.protocol'),
          options: protocolChoices(t).map((choice) => ({
            value: choice.value,
            label: choice.label,
          })),
        },
      ],
    },
    {
      key: 'redirects',
      header: t('list.columns.redirects'),
      align: 'right',
      render: (c) =>
        redirectCount(c) > 0 ? (
          <span className='tnum text-neutral-600 dark:text-neutral-400'>{redirectCount(c)}</span>
        ) : (
          <span className='tnum text-fk-amber'>0</span>
        ),
      filters: [
        {
          kind: 'boolean',
          key: 'has_redirect_uris',
          label: t('list.filter_fields.has_redirect_uris'),
        },
      ],
    },
    {
      key: 'status',
      header: t('list.columns.status'),
      render: (c) => (
        <span className='inline-flex items-center gap-1.5 text-xs text-neutral-600 dark:text-neutral-400'>
          <StatusDot on={c.enabled} />
          {t(`shared.status.${clientStateOf(c.enabled)}`)}
        </span>
      ),
      sortKey: 'enabled',
      filters: [
        { kind: 'boolean', key: 'enabled', label: t('list.filter_fields.enabled') },
        {
          kind: 'boolean',
          key: 'maintenance_enabled',
          label: t('list.filter_fields.maintenance_enabled'),
        },
      ],
    },
    {
      key: 'created',
      header: t('list.columns.created'),
      render: (c) => (
        <span className='tnum text-neutral-600 dark:text-neutral-400'>
          {formatRelative(c.created_at)}
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
      render: (c) => (
        <span className='tnum text-neutral-600 dark:text-neutral-400'>
          {formatRelative(c.updated_at)}
        </span>
      ),
      sortKey: 'updated_at',
    },
  ]

  const card: CardSpec<Client> = {
    avatar: (c) => <Squircle name={c.name || c.client_id} />,
    title: (c) => c.name,
    subtitle: (c) => c.client_id,
    badges: (c) => (
      <>
        <Pill tone={c.public_client ? 'info' : 'violet'} mono>
          {t(`shared.authentication.${clientAuthenticationOf(c.public_client)}`)}
        </Pill>
        <Pill tone='primary' mono>
          {c.protocol}
        </Pill>
        <Pill tone={c.enabled ? 'success' : 'neutral'}>
          <StatusDot on={c.enabled} />
          {t(`shared.state.${clientStateOf(c.enabled)}`)}
        </Pill>
      </>
    ),
    flags: (c) => [
      { label: t('list.card.flags.direct_access_grants'), on: c.direct_access_grants_enabled },
      { label: t('list.card.flags.device_code_grant'), on: c.oauth_device_code_grant_enabled },
      { label: t('list.card.flags.service_account'), on: c.service_account_enabled },
      { label: t('list.card.flags.require_pkce'), on: c.require_pkce },
    ],
    footer: (c) => (
      <>
        <span className='tnum'>{t('list.card.redirects', { count: redirectCount(c) })}</span>
        {c.maintenance_enabled && (
          <span className='text-fk-amber'>{t('shared.in_maintenance')}</span>
        )}
      </>
    ),
  }

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
            tone: 'info',
            filter: {},
          },
          {
            key: 'active',
            label: t('list.metrics.active.label'),
            value: counts.active,
            hint: t('list.metrics.active.hint'),
            series: [counts.active, counts.active],
            tone: 'success',
            filter: { enabled: YES },
          },
          {
            key: 'public',
            label: t('list.metrics.public.label'),
            value: counts.public,
            hint: t('list.metrics.public.hint'),
            series: [counts.public, counts.public],
            tone: 'info',
            filter: { public_client: YES },
          },
          {
            key: 'confidential',
            label: t('list.metrics.confidential.label'),
            value: counts.confidential,
            hint: t('list.metrics.confidential.hint'),
            series: [counts.confidential, counts.confidential],
            tone: 'violet',
            filter: { public_client: NO },
          },
        ]}
        alerts={[
          ...(withoutRedirect.total
            ? [
                {
                  tone: 'warn' as const,
                  title: t('list.alerts.no_redirect.title', { count: withoutRedirect.total }),
                  detail: t('list.alerts.no_redirect.detail', {
                    clients: previewNames(withoutRedirect),
                  }),
                  action: t('list.alerts.no_redirect.action'),
                },
              ]
            : []),
          ...(inMaintenance.total
            ? [
                {
                  tone: 'warn' as const,
                  title: t('list.alerts.maintenance.title', { count: inMaintenance.total }),
                  detail: t('list.alerts.maintenance.detail', {
                    clients: previewNames(inMaintenance),
                  }),
                  action: t('list.alerts.maintenance.action'),
                },
              ]
            : []),
        ]}
        paged={{ listing, pagination, search: { placeholder: t('list.search_placeholder') } }}
        rows={clients}
        columns={columns}
        card={card}
        getKey={(c) => c.id}
        getHref={clientHref}
        aggregates={{
          name: t('list.aggregates.count', { count: counts.total }),
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
        options={protocolChoices(t)}
        defaultValue={DEFAULT_PROTOCOL}
        createUrl={createUrl}
      />
    </>
  )
}
