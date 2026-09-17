import { Plus } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/kit/button'
import {
  CreatePickerDialog,
  ListingPage,
  Pill,
  Squircle,
  StatusDot,
  type CardSpec,
  type Column,
} from '@/components/kit'
import { Schemas } from '@/api/api.client'
import {
  DEFAULT_PROTOCOL,
  clientAuthenticationOf,
  clientStateOf,
  protocolChoices,
  type ClientProtocol,
} from '../client-choices'

import Client = Schemas.Client
import { cumulativeSeries } from '@/utils/cumulative-series'

export interface PageClientsOverviewProps {
  clients: Client[]
  isLoading: boolean
  pickerOpen: boolean
  onPickerOpenChange: (open: boolean) => void
  createUrl: (protocol: ClientProtocol) => string
  clientHref: (client: Client) => string
}

const redirectCount = (client: Client) => client.redirect_uris?.length ?? 0

export default function PageClientsOverview({
  clients,
  isLoading,
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
      sortValue: (c) => c.name,
    },
    {
      key: 'client_id',
      header: t('list.columns.client_id'),
      render: (c) => <span className='font-mono-ui text-xs text-neutral-500 dark:text-neutral-400'>{c.client_id}</span>,
      sortValue: (c) => c.client_id,
    },
    {
      key: 'kind',
      header: t('list.columns.kind'),
      render: (c) => (
        <Pill tone={c.public_client ? 'info' : 'violet'} mono>
          {t(`shared.authentication.${clientAuthenticationOf(c.public_client)}`)}
        </Pill>
      ),
      sortValue: (c) => clientAuthenticationOf(c.public_client),
    },
    {
      key: 'protocol',
      header: t('list.columns.protocol'),
      render: (c) => (
        <Pill tone='primary' mono>
          {c.protocol}
        </Pill>
      ),
      sortValue: (c) => c.protocol,
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
      sortValue: (c) => redirectCount(c),
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
      sortValue: (c) => clientStateOf(c.enabled),
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

  const publicClients = clients.filter((c) => c.public_client)
  const confidentialClients = clients.filter((c) => !c.public_client)
  const activeClients = clients.filter((c) => c.enabled)
  const withoutRedirect = clients.filter((c) => redirectCount(c) === 0)
  const inMaintenance = clients.filter((c) => c.maintenance_enabled)

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
            value: clients.length,
            hint: t('list.metrics.total.hint'),
            series: cumulativeSeries(clients.map((c) => c.created_at)),
            tone: 'info',
          },
          {
            key: 'active',
            label: t('list.metrics.active.label'),
            value: activeClients.length,
            hint: t('list.metrics.active.hint'),
            series: cumulativeSeries(activeClients.map((c) => c.created_at)),
            tone: 'success',
          },
          {
            key: 'public',
            label: t('list.metrics.public.label'),
            value: publicClients.length,
            hint: t('list.metrics.public.hint'),
            series: cumulativeSeries(publicClients.map((c) => c.created_at)),
            tone: 'info',
          },
          {
            key: 'confidential',
            label: t('list.metrics.confidential.label'),
            value: confidentialClients.length,
            hint: t('list.metrics.confidential.hint'),
            series: cumulativeSeries(confidentialClients.map((c) => c.created_at)),
            tone: 'violet',
          },
        ]}
        alerts={[
          ...(withoutRedirect.length
            ? [
                {
                  tone: 'warn' as const,
                  title: t('list.alerts.no_redirect.title', { count: withoutRedirect.length }),
                  detail: t('list.alerts.no_redirect.detail', {
                    clients: withoutRedirect.map((c) => c.name || c.client_id).join(', '),
                  }),
                  action: t('list.alerts.no_redirect.action'),
                },
              ]
            : []),
          ...(inMaintenance.length
            ? [
                {
                  tone: 'warn' as const,
                  title: t('list.alerts.maintenance.title', { count: inMaintenance.length }),
                  detail: t('list.alerts.maintenance.detail', {
                    clients: inMaintenance.map((c) => c.name || c.client_id).join(', '),
                  }),
                  action: t('list.alerts.maintenance.action'),
                },
              ]
            : []),
        ]}
        filters={[
          { key: 'public', label: t('list.filters.public'), predicate: (c) => c.public_client },
          {
            key: 'confidential',
            label: t('list.filters.confidential'),
            predicate: (c) => !c.public_client,
          },
          { key: 'disabled', label: t('list.filters.disabled'), predicate: (c) => !c.enabled },
        ]}
        searchPlaceholder={t('list.search_placeholder')}
        querySyntax={t('list.query_syntax')}
        searchIn={(c) => `${c.name} ${c.client_id}`}
        rows={clients}
        columns={columns}
        card={card}
        getKey={(c) => c.id}
        getHref={clientHref}
        aggregates={{
          name: t('list.aggregates.count', { count: clients.length }),
          redirects: clients.reduce((n, c) => n + redirectCount(c), 0),
          status: t('list.aggregates.enabled', { total: activeClients.length }),
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
