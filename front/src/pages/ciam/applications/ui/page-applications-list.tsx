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
import { formatDate } from '@/utils/format-date'
import { Schemas } from '@/api/api.client'
import {
  APPLICATION_TYPE_METAS,
  applicationTypeChoices,
  applicationTypeMeta,
  inferApplicationType,
  type ApplicationType,
} from '../application-types'

import Client = Schemas.Client

export interface PageApplicationsListProps {
  applications: Client[]
  isLoading: boolean
  pickerOpen: boolean
  onPickerOpenChange: (open: boolean) => void
  createUrl: (type: ApplicationType) => string
  applicationHref: (application: Client) => string
}

const callbackCount = (application: Client) => application.redirect_uris?.length ?? 0

const displayName = (application: Client) => application.name || application.client_id

const needsCallback = (application: Client) =>
  applicationTypeMeta(inferApplicationType(application)).usesAuthorizationCode &&
  callbackCount(application) === 0

export default function PageApplicationsList({
  applications,
  isLoading,
  pickerOpen,
  onPickerOpenChange,
  createUrl,
  applicationHref,
}: PageApplicationsListProps) {
  const columns: Column<Client>[] = [
    {
      key: 'name',
      header: 'Application',
      render: (c) => displayName(c),
      sortValue: (c) => displayName(c),
    },
    {
      key: 'client_id',
      header: 'client_id',
      render: (c) => (
        <span className='font-mono-ui text-xs text-neutral-500 dark:text-neutral-400'>
          {c.client_id}
        </span>
      ),
      sortValue: (c) => c.client_id,
    },
    {
      key: 'type',
      header: 'Type',
      render: (c) => {
        const meta = applicationTypeMeta(inferApplicationType(c))
        return <Pill tone={meta.tone}>{meta.short}</Pill>
      },
      sortValue: (c) => applicationTypeMeta(inferApplicationType(c)).short,
    },
    {
      key: 'flow',
      header: 'Sign-in flow',
      render: (c) => (
        <span className='text-neutral-500 dark:text-neutral-400'>
          {applicationTypeMeta(inferApplicationType(c)).flow}
        </span>
      ),
      sortValue: (c) => applicationTypeMeta(inferApplicationType(c)).flow,
    },
    {
      key: 'callbacks',
      header: 'Callback URLs',
      align: 'right',
      render: (c) =>
        applicationTypeMeta(inferApplicationType(c)).usesAuthorizationCode ? (
          <span className={callbackCount(c) === 0 ? 'tnum text-fk-amber' : 'tnum text-neutral-600 dark:text-neutral-400'}>
            {callbackCount(c)}
          </span>
        ) : (
          <span className='text-neutral-300 dark:text-neutral-600'>—</span>
        ),
      sortValue: (c) => callbackCount(c),
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
    {
      key: 'created_at',
      header: 'Created',
      render: (c) => <span className='tnum text-neutral-500 dark:text-neutral-400'>{formatDate(c.created_at)}</span>,
      sortValue: (c) => c.created_at,
    },
  ]

  const card: CardSpec<Client> = {
    avatar: (c) => <Squircle name={displayName(c)} />,
    title: (c) => displayName(c),
    subtitle: (c) => c.client_id,
    badges: (c) => {
      const meta = applicationTypeMeta(inferApplicationType(c))
      return (
        <>
          <Pill tone={meta.tone}>{meta.short}</Pill>
          <Pill tone={c.enabled ? 'success' : 'neutral'}>
            <StatusDot on={c.enabled} />
            {c.enabled ? 'enabled' : 'disabled'}
          </Pill>
        </>
      )
    },
    flags: (c) => [
      { label: 'Holds a secret', on: Boolean(c.secret) },
      { label: 'PKCE required', on: c.require_pkce },
      { label: 'Device grant', on: c.oauth_device_code_grant_enabled },
      { label: 'Direct access grants', on: c.direct_access_grants_enabled },
    ],
    footer: (c) => (
      <>
        <span>{applicationTypeMeta(inferApplicationType(c)).flow}</span>
        <span className='tnum'>{formatDate(c.created_at)}</span>
      </>
    ),
  }

  const active = applications.filter((c) => c.enabled)
  const withSecret = applications.filter((c) => Boolean(c.secret))
  const missingCallback = applications.filter(needsCallback)

  const createButton = (
    <Button onClick={() => onPickerOpenChange(true)}>
      <Plus /> Create application
    </Button>
  )

  const typeFilters = APPLICATION_TYPE_METAS.map((meta) => ({
    key: meta.key,
    label: `${meta.short} (${applications.filter((c) => inferApplicationType(c) === meta.key).length})`,
    predicate: (c: Client) => inferApplicationType(c) === meta.key,
  }))

  return (
    <>
      <ListingPage
        title='Applications'
        description='Anything that signs your users in: mobile apps, single-page apps, web servers, daemons.'
        loading={isLoading}
        actions={createButton}
        metrics={[
          { key: 'total', label: 'Total', value: applications.length, hint: 'registered' },
          { key: 'active', label: 'Active', value: active.length, hint: 'can sign users in' },
          {
            key: 'secret',
            label: 'Hold a secret',
            value: withSecret.length,
            hint: 'must stay server-side',
          },
          {
            key: 'missing-callback',
            label: 'Missing a callback URL',
            value: missingCallback.length,
            hint: 'sign-in will fail',
          },
        ]}
        alerts={
          missingCallback.length
            ? [
                {
                  tone: 'warn' as const,
                  title: `${missingCallback.length} application${missingCallback.length > 1 ? 's have' : ' has'} no callback URL`,
                  detail: `${missingCallback.map(displayName).join(', ')} — FerrisKey has nowhere to send the user back, so sign-in stops with an error.`,
                },
              ]
            : []
        }
        filters={[
          ...typeFilters,
          { key: 'disabled', label: 'Disabled', predicate: (c: Client) => !c.enabled },
        ]}
        searchPlaceholder='Filter by name or client_id…'
        searchIn={(c) => `${c.name} ${c.client_id}`}
        rows={applications}
        columns={columns}
        card={card}
        getKey={(c) => c.id}
        getHref={applicationHref}
        aggregates={{
          name: `${applications.length} application${applications.length === 1 ? '' : 's'}`,
          callbacks: applications.reduce((n, c) => n + callbackCount(c), 0),
          status: `${active.length} enabled`,
        }}
        emptyLabel='No application yet'
        emptyHint='Pick a type, give it a name, and we generate everything you need to integrate.'
        emptyAction={createButton}
      />

      <CreatePickerDialog
        open={pickerOpen}
        onOpenChange={onPickerOpenChange}
        title='Application type'
        description='It decides which sign-in flow the application uses and whether it gets a secret. Changing it later means recreating the application.'
        options={applicationTypeChoices}
        defaultValue='spa'
        createUrl={createUrl}
      />
    </>
  )
}
