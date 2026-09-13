import { Plus } from 'lucide-react'
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
import { protocolChoices, type ClientProtocol } from '../client-choices'

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
  const columns: Column<Client>[] = [
    {
      key: 'name',
      header: 'Client',
      render: (c) => c.name,
      sortValue: (c) => c.name,
    },
    {
      key: 'client_id',
      header: 'client_id',
      render: (c) => <span className='font-mono-ui text-xs text-neutral-500 dark:text-neutral-400'>{c.client_id}</span>,
      sortValue: (c) => c.client_id,
    },
    {
      key: 'kind',
      header: 'Type',
      render: (c) => (
        <Pill tone={c.public_client ? 'info' : 'violet'} mono>
          {c.public_client ? 'public' : 'confidential'}
        </Pill>
      ),
      sortValue: (c) => (c.public_client ? 'public' : 'confidential'),
    },
    {
      key: 'protocol',
      header: 'Protocol',
      render: (c) => (
        <Pill tone='primary' mono>
          {c.protocol}
        </Pill>
      ),
      sortValue: (c) => c.protocol,
    },
    {
      key: 'redirects',
      header: 'Redirect URIs',
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
      header: 'Status',
      render: (c) => (
        <span className='inline-flex items-center gap-1.5 text-xs text-neutral-600 dark:text-neutral-400'>
          <StatusDot on={c.enabled} />
          {c.enabled ? 'Enabled' : 'Disabled'}
        </span>
      ),
      sortValue: (c) => (c.enabled ? 'enabled' : 'disabled'),
    },
  ]

  const card: CardSpec<Client> = {
    avatar: (c) => <Squircle name={c.name || c.client_id} />,
    title: (c) => c.name,
    subtitle: (c) => c.client_id,
    badges: (c) => (
      <>
        <Pill tone={c.public_client ? 'info' : 'violet'} mono>
          {c.public_client ? 'public' : 'confidential'}
        </Pill>
        <Pill tone='primary' mono>
          {c.protocol}
        </Pill>
        <Pill tone={c.enabled ? 'success' : 'neutral'}>
          <StatusDot on={c.enabled} />
          {c.enabled ? 'enabled' : 'disabled'}
        </Pill>
      </>
    ),
    flags: (c) => [
      { label: 'Direct access grants', on: c.direct_access_grants_enabled },
      { label: 'Device authorization grant', on: c.oauth_device_code_grant_enabled },
      { label: 'Service account', on: c.service_account_enabled },
      { label: 'PKCE required', on: c.require_pkce },
    ],
    footer: (c) => (
      <>
        <span className='tnum'>
          {redirectCount(c)} redirect URI{redirectCount(c) === 1 ? '' : 's'}
        </span>
        {c.maintenance_enabled && <span className='text-fk-amber'>in maintenance</span>}
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
      <Plus /> New client
    </Button>
  )

  return (
    <>
      <ListingPage
        title='Clients'
        description='Applications and services allowed to request an authentication.'
        loading={isLoading}
        actions={createButton}
        metrics={[
          {
            key: 'total',
            label: 'Total',
            value: clients.length,
            hint: 'registered clients',
            series: cumulativeSeries(clients.map((c) => c.created_at)),
            tone: 'info',
          },
          {
            key: 'active',
            label: 'Active',
            value: activeClients.length,
            hint: 'can authenticate',
            series: cumulativeSeries(activeClients.map((c) => c.created_at)),
            tone: 'success',
          },
          {
            key: 'public',
            label: 'Public',
            value: publicClients.length,
            hint: 'no secret',
            series: cumulativeSeries(publicClients.map((c) => c.created_at)),
            tone: 'info',
          },
          {
            key: 'confidential',
            label: 'Confidential',
            value: confidentialClients.length,
            hint: 'hold a secret',
            series: cumulativeSeries(confidentialClients.map((c) => c.created_at)),
            tone: 'violet',
          },
        ]}
        alerts={[
          ...(withoutRedirect.length
            ? [
                {
                  tone: 'warn' as const,
                  title: `${withoutRedirect.length} client${withoutRedirect.length > 1 ? 's have' : ' has'} no redirect URI`,
                  detail: `${withoutRedirect.map((c) => c.name || c.client_id).join(', ')} — the authorization code flow will fail.`,
                  action: 'Configure',
                },
              ]
            : []),
          ...(inMaintenance.length
            ? [
                {
                  tone: 'warn' as const,
                  title: `${inMaintenance.length} client${inMaintenance.length > 1 ? 's are' : ' is'} in maintenance`,
                  detail: `${inMaintenance.map((c) => c.name || c.client_id).join(', ')} — only whitelisted accounts can sign in.`,
                  action: 'Inspect',
                },
              ]
            : []),
        ]}
        filters={[
          { key: 'public', label: 'Public', predicate: (c) => c.public_client },
          { key: 'confidential', label: 'Confidential', predicate: (c) => !c.public_client },
          { key: 'disabled', label: 'Disabled', predicate: (c) => !c.enabled },
        ]}
        searchPlaceholder='Filter by name or client_id…'
        querySyntax='name:*portal  type:confidential  enabled:true'
        searchIn={(c) => `${c.name} ${c.client_id}`}
        rows={clients}
        columns={columns}
        card={card}
        getKey={(c) => c.id}
        getHref={clientHref}
        aggregates={{
          name: `${clients.length} client${clients.length === 1 ? '' : 's'}`,
          redirects: clients.reduce((n, c) => n + redirectCount(c), 0),
          status: `${activeClients.length} enabled`,
        }}
        emptyLabel='No client'
        emptyHint='Register an application to let it authenticate the accounts of this realm.'
        emptyAction={createButton}
      />

      <CreatePickerDialog
        open={pickerOpen}
        onOpenChange={onPickerOpenChange}
        title='Client protocol'
        description='It decides how the application signs its users in, and cannot be changed after creation.'
        options={protocolChoices}
        defaultValue='openid-connect'
        createUrl={createUrl}
      />
    </>
  )
}
