import { useState } from 'react'
import { KeyRound, Plus, Trash2 } from 'lucide-react'
import { Button } from '@/components/kit/button'
import { ConfirmDeleteAlert } from '@/components/confirm-delete-alert'
import { ListingPage, IconTile, Pill } from '@/components/kit'
import type { CardSpec, Column, PillTone } from '@/components/kit'
import { Schemas } from '@/api/api.client'

import ClientScope = Schemas.ClientScope
import ScopeType = Schemas.ScopeType
import { cumulativeSeries } from '@/shared/cumulative-series'

const typeLabel: Record<ScopeType, string> = {
  DEFAULT: 'default',
  OPTIONAL: 'optional',
  NONE: 'none',
}

const typeTone: Record<ScopeType, PillTone> = {
  DEFAULT: 'success',
  OPTIONAL: 'info',
  NONE: 'neutral',
}

const mapperCount = (scope: ClientScope) => scope.protocol_mappers?.length ?? 0

export interface PageClientScopesOverviewProps {
  scopes: ClientScope[]
  isLoading: boolean
  isDeleting: boolean
  scopeHref: (scope: ClientScope) => string
  onCreate: () => void
  onDelete: (scope: ClientScope) => void
}

export default function PageClientScopesOverview({
  scopes,
  isLoading,
  isDeleting,
  scopeHref,
  onCreate,
  onDelete,
}: PageClientScopesOverviewProps) {
  const [pendingDelete, setPendingDelete] = useState<ClientScope | null>(null)

  const columns: Column<ClientScope>[] = [
    {
      key: 'name',
      header: 'Scope',
      render: (s) => s.name,
      sortValue: (s) => s.name,
    },
    {
      key: 'description',
      header: 'Description',
      render: (s) =>
        s.description ? (
          <span className='text-neutral-600 dark:text-neutral-400'>{s.description}</span>
        ) : (
          <span className='font-mono-ui text-xs text-neutral-400 dark:text-neutral-500'>scope_id: {s.id}</span>
        ),
    },
    {
      key: 'type',
      header: 'Assignment',
      render: (s) => (
        <Pill tone={typeTone[s.default_scope_type]} mono>
          {typeLabel[s.default_scope_type]}
        </Pill>
      ),
      sortValue: (s) => typeLabel[s.default_scope_type],
    },
    {
      key: 'protocol',
      header: 'Protocol',
      render: (s) => (
        <Pill mono>{s.protocol}</Pill>
      ),
      sortValue: (s) => s.protocol,
    },
    {
      key: 'mappers',
      header: 'Mappers',
      align: 'right',
      render: (s) =>
        mapperCount(s) > 0 ? (
          <span className='tnum text-neutral-600 dark:text-neutral-400'>{mapperCount(s)}</span>
        ) : (
          <span className='tnum text-neutral-300 dark:text-neutral-600'>0</span>
        ),
      sortValue: (s) => mapperCount(s),
    },
    {
      key: 'actions',
      header: '',
      align: 'right',
      render: (s) => (
        <Button
          variant='ghost'
          size='icon'
          aria-label={`Delete ${s.name}`}
          onClick={() => setPendingDelete(s)}
          className='size-7 text-neutral-400 dark:text-neutral-500 hover:text-fk-danger'
        >
          <Trash2 />
        </Button>
      ),
    },
  ]

  const card: CardSpec<ClientScope> = {
    avatar: () => (
      <IconTile tone='info'>
        <KeyRound className='size-4' strokeWidth={1.75} />
      </IconTile>
    ),
    title: (s) => s.name,
    subtitle: (s) => s.protocol,
    badges: (s) => (
      <>
        <Pill tone={typeTone[s.default_scope_type]} mono>
          {typeLabel[s.default_scope_type]}
        </Pill>
        <Pill tone={mapperCount(s) > 0 ? 'success' : 'amber'}>
          {mapperCount(s)} mapper{mapperCount(s) !== 1 ? 's' : ''}
        </Pill>
      </>
    ),
    flags: (s) => [
      { label: 'Granted to every client by default', on: s.default_scope_type === 'DEFAULT' },
      { label: 'Writes at least one claim', on: mapperCount(s) > 0 },
    ],
    footer: (s) => (
      <>
        <span className='tnum'>{mapperCount(s)} mappers</span>
        <span className='truncate pl-3 text-right'>
          {s.description || `scope_id: ${s.id}`}
        </span>
      </>
    ),
  }

  const defaultScopes = scopes.filter((s) => s.default_scope_type === 'DEFAULT')
  const optionalScopes = scopes.filter((s) => s.default_scope_type === 'OPTIONAL')
  const withMappers = scopes.filter((s) => mapperCount(s) > 0)
  const withoutMappers = scopes.filter((s) => mapperCount(s) === 0)

  const createButton = (
    <Button onClick={onCreate}>
      <Plus /> New client scope
    </Button>
  )

  return (
    <>
      <ListingPage
        title='Client Scopes'
        description='Reusable sets of claims that clients of this realm request by name.'
        loading={isLoading}
        actions={createButton}
        metrics={[
          { key: 'total', label: 'Total', value: scopes.length, hint: 'scopes', series: cumulativeSeries(scopes.map((r) => r.created_at)), tone: 'info' },
          {
            key: 'default',
            label: 'Default scopes',
            value: defaultScopes.length,
            hint:
              defaultScopes.length > 0 && scopes.length > 0
                ? `${((defaultScopes.length / scopes.length) * 100).toFixed(0)}% of total`
                : 'no default scope',
            series: cumulativeSeries(defaultScopes.map((r) => r.created_at)),
            tone: 'success',
          },
          {
            key: 'optional',
            label: 'Optional scopes',
            value: optionalScopes.length,
            hint: 'granted on request',
            series: cumulativeSeries(optionalScopes.map((r) => r.created_at)),
            tone: 'violet',
          },
          {
            key: 'mappers',
            label: 'With mappers',
            value: withMappers.length,
            hint: 'write at least one claim',
            series: cumulativeSeries(withMappers.map((r) => r.created_at)),
            tone: 'info',
          },
        ]}
        alerts={
          withoutMappers.length
            ? [
                {
                  tone: 'warn' as const,
                  title: `${withoutMappers.length} scope${withoutMappers.length > 1 ? 's write' : ' writes'} no claim`,
                  detail: `${withoutMappers.map((s) => s.name).join(', ')} — carries no protocol mapper.`,
                  action: 'Review',
                },
              ]
            : []
        }
        filters={[
          { key: 'default', label: 'Default', predicate: (s) => s.default_scope_type === 'DEFAULT' },
          {
            key: 'optional',
            label: 'Optional',
            predicate: (s) => s.default_scope_type === 'OPTIONAL',
          },
          { key: 'empty', label: 'Without mappers', predicate: (s) => mapperCount(s) === 0 },
        ]}
        searchPlaceholder='Filter by name…'
        querySyntax='name:profile*  type:optional  protocol:openid-connect'
        searchIn={(s) => `${s.name} ${s.description ?? ''} ${s.protocol}`}
        rows={scopes}
        columns={columns}
        card={card}
        getKey={(s) => s.id}
        getHref={scopeHref}
        aggregates={{
          name: `${scopes.length} scope${scopes.length !== 1 ? 's' : ''}`,
          mappers: scopes.reduce((n, s) => n + mapperCount(s), 0),
        }}
        emptyLabel='No client scope'
        emptyHint='A client scope groups the claims a client can ask for in its tokens.'
        emptyAction={createButton}
      />

      <ConfirmDeleteAlert
        open={Boolean(pendingDelete)}
        title='Delete client scope'
        description={
          pendingDelete
            ? `This will permanently delete "${pendingDelete.name}" and all its associated protocol mappers and client mappings.`
            : ''
        }
        onConfirm={() => {
          if (!pendingDelete || isDeleting) return
          onDelete(pendingDelete)
          setPendingDelete(null)
        }}
        onCancel={() => setPendingDelete(null)}
      />
    </>
  )
}
