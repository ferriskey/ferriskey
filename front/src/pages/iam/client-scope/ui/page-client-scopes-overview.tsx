import { useState } from 'react'
import { KeyRound, Plus, Trash2 } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/kit/button'
import { ConfirmDeleteAlert } from '@/components/confirm-delete-alert'
import { ListingPage, IconTile, Pill } from '@/components/kit'
import type {
  CardSpec,
  Column,
  PagedListing,
  PaginationMetadata,
} from '@/components/kit'
import { Schemas } from '@/api/api.client'
import { formatRelative } from '@/utils/format-date'

import ClientScope = Schemas.ClientScope
import ScopeType = Schemas.ScopeType
import { SCOPE_TYPE_TONE, scopeTypeLabelKey } from '../scope-type'

const SCOPE_TYPES: ScopeType[] = ['DEFAULT', 'OPTIONAL', 'NONE']
const PROTOCOLS = ['openid-connect', 'saml'] as const
const NAME_SEPARATOR = ', '
const TRUNCATION_MARK = '…'
const YES = 'true'

const mapperCount = (scope: ClientScope) => scope.protocol_mappers?.length ?? 0

export interface ClientScopeCounts {
  total: number
  default: number
  optional: number
  withMappers: number
}

export interface ClientScopePreview {
  total: number
  names: string[]
}

const previewNames = (preview: ClientScopePreview) =>
  preview.names.join(NAME_SEPARATOR) +
  (preview.total > preview.names.length ? TRUNCATION_MARK : '')

export interface PageClientScopesOverviewProps {
  scopes: ClientScope[]
  pagination: PaginationMetadata | undefined
  listing: PagedListing
  isLoading: boolean
  isDeleting: boolean
  counts: ClientScopeCounts
  withoutMappers: ClientScopePreview
  scopeHref: (scope: ClientScope) => string
  onCreate: () => void
  onDelete: (scope: ClientScope) => void
}

export default function PageClientScopesOverview({
  scopes,
  pagination,
  listing,
  isLoading,
  isDeleting,
  counts,
  withoutMappers,
  scopeHref,
  onCreate,
  onDelete,
}: PageClientScopesOverviewProps) {
  const { t } = useTranslation('client-scope')
  const [pendingDelete, setPendingDelete] = useState<ClientScope | null>(null)

  const columns: Column<ClientScope>[] = [
    {
      key: 'name',
      header: t('list.columns.name'),
      render: (s) => s.name,
      sortKey: 'name',
      filters: [{ kind: 'text', key: 'name', label: t('list.filter_fields.name') }],
    },
    {
      key: 'description',
      header: t('list.columns.description'),
      render: (s) =>
        s.description ? (
          <span className='text-neutral-600 dark:text-neutral-400'>{s.description}</span>
        ) : (
          <span className='font-mono-ui text-xs text-neutral-400 dark:text-neutral-500'>
            {t('scope.identifier', { id: s.id })}
          </span>
        ),
      filters: [{ kind: 'text', key: 'description', label: t('list.filter_fields.description') }],
    },
    {
      key: 'type',
      header: t('list.columns.type'),
      render: (s) => (
        <Pill tone={SCOPE_TYPE_TONE[s.default_scope_type]} mono>
          {t(scopeTypeLabelKey(s.default_scope_type))}
        </Pill>
      ),
      filters: [
        {
          kind: 'enum',
          key: 'default_scope_type',
          label: t('list.filter_fields.default_scope_type'),
          options: SCOPE_TYPES.map((scopeType) => ({
            value: scopeType,
            label: t(scopeTypeLabelKey(scopeType)),
          })),
        },
      ],
    },
    {
      key: 'protocol',
      header: t('list.columns.protocol'),
      render: (s) => (
        <Pill mono>{s.protocol}</Pill>
      ),
      filters: [
        {
          kind: 'enum',
          key: 'protocol',
          label: t('list.filter_fields.protocol'),
          options: PROTOCOLS.map((protocol) => ({ value: protocol, label: protocol })),
        },
      ],
    },
    {
      key: 'mappers',
      header: t('list.columns.mappers'),
      align: 'right',
      render: (s) =>
        mapperCount(s) > 0 ? (
          <span className='tnum text-neutral-600 dark:text-neutral-400'>{mapperCount(s)}</span>
        ) : (
          <span className='tnum text-neutral-300 dark:text-neutral-600'>0</span>
        ),
      filters: [
        {
          kind: 'boolean',
          key: 'has_protocol_mappers',
          label: t('list.filter_fields.has_protocol_mappers'),
        },
      ],
    },
    {
      key: 'created',
      header: t('list.columns.created'),
      render: (s) => (
        <span className='tnum text-neutral-600 dark:text-neutral-400'>
          {formatRelative(s.created_at)}
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
      render: (s) => (
        <span className='tnum text-neutral-600 dark:text-neutral-400'>
          {formatRelative(s.updated_at)}
        </span>
      ),
      sortKey: 'updated_at',
    },
    {
      key: 'actions',
      header: '',
      align: 'right',
      render: (s) => (
        <Button
          variant='ghost'
          size='icon'
          aria-label={t('list.row_delete', { name: s.name })}
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
        <Pill tone={SCOPE_TYPE_TONE[s.default_scope_type]} mono>
          {t(scopeTypeLabelKey(s.default_scope_type))}
        </Pill>
        <Pill tone={mapperCount(s) > 0 ? 'success' : 'amber'}>
          {t('scope.mapper_count', { count: mapperCount(s) })}
        </Pill>
      </>
    ),
    flags: (s) => [
      { label: t('list.card.flags.default'), on: s.default_scope_type === 'DEFAULT' },
      { label: t('list.card.flags.has_mappers'), on: mapperCount(s) > 0 },
    ],
    footer: (s) => (
      <>
        <span className='tnum'>{t('list.card.footer_mappers', { count: mapperCount(s) })}</span>
        <span className='truncate pl-3 text-right'>
          {s.description || t('scope.identifier', { id: s.id })}
        </span>
      </>
    ),
  }

  const createButton = (
    <Button onClick={onCreate}>
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
            onSelect: () =>
              listing.setFilters({ default_scope_type: '', has_protocol_mappers: '' }),
          },
          {
            key: 'default',
            label: t('list.metrics.default.label'),
            value: counts.default,
            hint:
              counts.default > 0 && counts.total > 0
                ? t('list.metrics.default.hint', {
                    percent: ((counts.default / counts.total) * 100).toFixed(0),
                  })
                : t('list.metrics.default.empty_hint'),
            series: [counts.default, counts.default],
            tone: 'success',
            onSelect: () => listing.setFilters({ default_scope_type: 'DEFAULT' }),
          },
          {
            key: 'optional',
            label: t('list.metrics.optional.label'),
            value: counts.optional,
            hint: t('list.metrics.optional.hint'),
            series: [counts.optional, counts.optional],
            tone: 'violet',
            onSelect: () => listing.setFilters({ default_scope_type: 'OPTIONAL' }),
          },
          {
            key: 'mappers',
            label: t('list.metrics.mappers.label'),
            value: counts.withMappers,
            hint: t('list.metrics.mappers.hint'),
            series: [counts.withMappers, counts.withMappers],
            tone: 'info',
            onSelect: () => listing.setFilters({ has_protocol_mappers: YES }),
          },
        ]}
        alerts={
          withoutMappers.total
            ? [
                {
                  tone: 'warn' as const,
                  title: t('list.alerts.without_mappers.title', { count: withoutMappers.total }),
                  detail: t('list.alerts.without_mappers.detail', {
                    names: previewNames(withoutMappers),
                  }),
                  action: t('list.alerts.without_mappers.action'),
                },
              ]
            : []
        }
        paged={{ listing, pagination, search: { placeholder: t('list.search_placeholder') } }}
        rows={scopes}
        columns={columns}
        card={card}
        getKey={(s) => s.id}
        getHref={scopeHref}
        aggregates={{
          name: t('list.count', { count: counts.total }),
        }}
        emptyLabel={t('list.empty.label')}
        emptyHint={t('list.empty.hint')}
        emptyAction={createButton}
      />

      <ConfirmDeleteAlert
        open={Boolean(pendingDelete)}
        title={t('list.delete.title')}
        description={
          pendingDelete ? t('list.delete.description', { name: pendingDelete.name }) : ''
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
