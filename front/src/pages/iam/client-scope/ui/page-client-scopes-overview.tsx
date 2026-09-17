import { useState } from 'react'
import { KeyRound, Plus, Trash2 } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/kit/button'
import { ConfirmDeleteAlert } from '@/components/confirm-delete-alert'
import { ListingPage, IconTile, Pill } from '@/components/kit'
import type { CardSpec, Column } from '@/components/kit'
import { Schemas } from '@/api/api.client'

import ClientScope = Schemas.ClientScope
import { cumulativeSeries } from '@/utils/cumulative-series'
import { SCOPE_TYPE_TONE, scopeTypeLabelKey } from '../scope-type'

const QUERY_SYNTAX = 'name:profile*  type:optional  protocol:openid-connect'

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
  const { t } = useTranslation('client-scope')
  const [pendingDelete, setPendingDelete] = useState<ClientScope | null>(null)

  const columns: Column<ClientScope>[] = [
    {
      key: 'name',
      header: t('list.columns.name'),
      render: (s) => s.name,
      sortValue: (s) => s.name,
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
    },
    {
      key: 'type',
      header: t('list.columns.type'),
      render: (s) => (
        <Pill tone={SCOPE_TYPE_TONE[s.default_scope_type]} mono>
          {t(scopeTypeLabelKey(s.default_scope_type))}
        </Pill>
      ),
      sortValue: (s) => t(scopeTypeLabelKey(s.default_scope_type)),
    },
    {
      key: 'protocol',
      header: t('list.columns.protocol'),
      render: (s) => (
        <Pill mono>{s.protocol}</Pill>
      ),
      sortValue: (s) => s.protocol,
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

  const defaultScopes = scopes.filter((s) => s.default_scope_type === 'DEFAULT')
  const optionalScopes = scopes.filter((s) => s.default_scope_type === 'OPTIONAL')
  const withMappers = scopes.filter((s) => mapperCount(s) > 0)
  const withoutMappers = scopes.filter((s) => mapperCount(s) === 0)

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
            value: scopes.length,
            hint: t('list.metrics.total.hint'),
            series: cumulativeSeries(scopes.map((r) => r.created_at)),
            tone: 'info',
          },
          {
            key: 'default',
            label: t('list.metrics.default.label'),
            value: defaultScopes.length,
            hint:
              defaultScopes.length > 0 && scopes.length > 0
                ? t('list.metrics.default.hint', {
                    percent: ((defaultScopes.length / scopes.length) * 100).toFixed(0),
                  })
                : t('list.metrics.default.empty_hint'),
            series: cumulativeSeries(defaultScopes.map((r) => r.created_at)),
            tone: 'success',
          },
          {
            key: 'optional',
            label: t('list.metrics.optional.label'),
            value: optionalScopes.length,
            hint: t('list.metrics.optional.hint'),
            series: cumulativeSeries(optionalScopes.map((r) => r.created_at)),
            tone: 'violet',
          },
          {
            key: 'mappers',
            label: t('list.metrics.mappers.label'),
            value: withMappers.length,
            hint: t('list.metrics.mappers.hint'),
            series: cumulativeSeries(withMappers.map((r) => r.created_at)),
            tone: 'info',
          },
        ]}
        alerts={
          withoutMappers.length
            ? [
                {
                  tone: 'warn' as const,
                  title: t('list.alerts.without_mappers.title', { count: withoutMappers.length }),
                  detail: t('list.alerts.without_mappers.detail', {
                    names: withoutMappers.map((s) => s.name).join(', '),
                  }),
                  action: t('list.alerts.without_mappers.action'),
                },
              ]
            : []
        }
        filters={[
          {
            key: 'default',
            label: t('list.filters.default'),
            predicate: (s) => s.default_scope_type === 'DEFAULT',
          },
          {
            key: 'optional',
            label: t('list.filters.optional'),
            predicate: (s) => s.default_scope_type === 'OPTIONAL',
          },
          {
            key: 'empty',
            label: t('list.filters.without_mappers'),
            predicate: (s) => mapperCount(s) === 0,
          },
        ]}
        searchPlaceholder={t('list.search_placeholder')}
        querySyntax={QUERY_SYNTAX}
        searchIn={(s) => `${s.name} ${s.description ?? ''} ${s.protocol}`}
        rows={scopes}
        columns={columns}
        card={card}
        getKey={(s) => s.id}
        getHref={scopeHref}
        aggregates={{
          name: t('list.count', { count: scopes.length }),
          mappers: scopes.reduce((n, s) => n + mapperCount(s), 0),
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
