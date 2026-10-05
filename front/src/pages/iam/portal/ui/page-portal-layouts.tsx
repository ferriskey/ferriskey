import { Trans, useTranslation } from 'react-i18next'
import { Download, LayoutTemplate, Plus, Trash2 } from 'lucide-react'
import { Button } from '@/components/kit/button'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
import {
  DataView,
  FilterBar,
  IconTile,
  MetricsBand,
  PageShell,
  PaginationBar,
  Pill,
  Section,
  type Column,
  type FilterField,
  type PagedListing,
  type PaginationMetadata,
  type ViewMode,
} from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import type { Schemas } from '@/api/api.client'
import { useThemesUsingLayout } from '@/api/portal-theme.api'
import { PORTAL_TAB_LAYOUTS, PortalPageHeader } from './portal-page-header'
import { ImportButton } from './import-button'
import { formatDate } from '@/utils/format-date'

export interface PortalLayoutRow {
  layout: Schemas.PortalLayoutListItem
  nodes: number
}

export interface PortalLayoutCounts {
  total: number
  defaults: number
  used: number
  free: number
}

export interface PagePortalLayoutsProps {
  realm: string
  rows: PortalLayoutRow[]
  listing: PagedListing
  pagination: PaginationMetadata | undefined
  counts: PortalLayoutCounts
  isLoading: boolean
  onCreate: () => void
  onEdit: (layoutId: string) => void
  onDelete: (layoutId: string) => void
  onExport: (layoutId: string) => void
  onImport: (file: File) => void
}

const LAYOUTS_VIEW: ViewMode = 'list'
const NAME_SEPARATOR = ', '
const TRUNCATION_MARK = '…'

function refusalKeyFor({ layout }: PortalLayoutRow): string | null {
  if (layout.is_default) return 'layouts.row.refusal.default'
  if (layout.theme_count > 0) return 'layouts.row.refusal.in_use'
  return null
}

function UsedByRefusal({
  realm,
  layoutId,
  count,
}: {
  realm: string
  layoutId: string
  count: number
}) {
  const { t } = useTranslation('portal')
  const { names, total } = useThemesUsingLayout({ realm, layoutId })
  const themes =
    names.length > 0
      ? names.join(NAME_SEPARATOR) + (total > names.length ? TRUNCATION_MARK : '')
      : t('layouts.row.theme_count', { count })
  return <>{t('layouts.row.refusal.in_use', { themes, count })}</>
}

export default function PagePortalLayouts({
  realm,
  rows,
  listing,
  pagination,
  counts,
  isLoading,
  onCreate,
  onEdit,
  onDelete,
  onExport,
  onImport,
}: PagePortalLayoutsProps) {
  const { t } = useTranslation('portal')

  const narrowed = Object.values(listing.state.filters).some(Boolean)

  const createButton = (
    <Button onClick={onCreate}>
      <Plus /> {t('layouts.create')}
    </Button>
  )

  const filterFields: FilterField[] = [
    { kind: 'text', key: 'name', label: t('layouts.filters.name') },
    { kind: 'boolean', key: 'is_default', label: t('layouts.filters.is_default') },
    { kind: 'boolean', key: 'in_use', label: t('layouts.filters.in_use') },
  ]

  const defaultPill = <Pill tone='violet'>{t('layouts.row.default')}</Pill>

  const usagePill = (layout: Schemas.PortalLayoutListItem) => (
    <Pill tone={layout.theme_count > 0 ? 'info' : 'neutral'}>
      {layout.theme_count > 0
        ? t('layouts.row.theme_count', { count: layout.theme_count })
        : t('layouts.row.unused')}
    </Pill>
  )

  const columns: Column<PortalLayoutRow>[] = [
    {
      key: 'name',
      header: t('layouts.columns.name'),
      sortKey: 'name',
      render: ({ layout, nodes }) => (
        <div className='flex min-w-0 items-center gap-3'>
          <IconTile tone={layout.is_default ? 'violet' : 'info'}>
            <LayoutTemplate className='size-4' strokeWidth={1.75} />
          </IconTile>
          <div className='min-w-0'>
            <div className='flex flex-wrap items-center gap-2'>
              <button
                type='button'
                onClick={() => onEdit(layout.id)}
                className='cursor-pointer text-[13px] font-medium text-neutral-900 dark:text-neutral-100 hover:underline'
              >
                {layout.name}
              </button>
              {layout.is_default && defaultPill}
              {usagePill(layout)}
            </div>
            <p className='mt-0.5 truncate text-xs text-neutral-500 dark:text-neutral-400'>
              <Trans
                i18nKey='portal:layouts.row.summary'
                count={nodes}
                values={{ date: formatDate(layout.updated_at) }}
                components={{ num: <span className='tnum' /> }}
              />
            </p>
            <p className='font-mono-ui mt-0.5 truncate text-[11px] text-neutral-400 dark:text-neutral-500'>
              {t('layouts.row.identifier', { id: layout.id })}
            </p>
          </div>
        </div>
      ),
    },
    {
      key: 'updated_at',
      header: t('layouts.columns.updated_at'),
      sortKey: 'updated_at',
      render: ({ layout }) => (
        <span className='tnum text-xs text-neutral-500 dark:text-neutral-400'>
          {formatDate(layout.updated_at)}
        </span>
      ),
    },
    {
      key: 'created_at',
      header: t('layouts.columns.created_at'),
      sortKey: 'created_at',
      render: ({ layout }) => (
        <span className='tnum text-xs text-neutral-500 dark:text-neutral-400'>
          {formatDate(layout.created_at)}
        </span>
      ),
    },
    {
      key: 'actions',
      header: '',
      align: 'right',
      render: (row) => {
        const { layout } = row
        const refusalKey = refusalKeyFor(row)
        return (
          <div className='flex items-center justify-end gap-1'>
            <Button variant='outline' size='sm' onClick={() => onEdit(layout.id)}>
              {t('layouts.row.open')}
            </Button>

            <Button
              variant='ghost'
              size='icon'
              className='size-8 text-neutral-400 dark:text-neutral-500'
              aria-label={t('layouts.row.export', { name: layout.name })}
              onClick={() => onExport(layout.id)}
            >
              <Download className='size-4' />
            </Button>

            {refusalKey ? (
              <Tooltip>
                <TooltipTrigger asChild>
                  <span className='grid size-8 place-items-center text-neutral-200'>
                    <Trash2 className='size-4' />
                  </span>
                </TooltipTrigger>
                <TooltipContent side='left' className='max-w-xs'>
                  {layout.is_default ? (
                    t(refusalKey)
                  ) : (
                    <UsedByRefusal
                      realm={realm}
                      layoutId={layout.id}
                      count={layout.theme_count}
                    />
                  )}
                </TooltipContent>
              </Tooltip>
            ) : (
              <Button
                variant='ghost'
                size='icon'
                aria-label={t('layouts.row.delete', { name: layout.name })}
                className='size-8 text-neutral-400 dark:text-neutral-500 hover:text-fk-danger'
                onClick={() => onDelete(layout.id)}
              >
                <Trash2 className='size-4' />
              </Button>
            )}
          </div>
        )
      },
    },
  ]

  return (
    <PageShell>
      <PortalPageHeader tab={PORTAL_TAB_LAYOUTS} actions={createButton} />

      <div className={cn('mt-4', tokens.page.blockGap)}>
        <MetricsBand
          metrics={[
            {
              key: 'total',
              label: t('layouts.metrics.total.label'),
              value: counts.total,
              hint: t('layouts.metrics.total.hint'),
              series: [counts.total, counts.total],
            },
            {
              key: 'default',
              label: t('layouts.metrics.default.label'),
              value: counts.defaults,
              hint: t('layouts.metrics.default.hint'),
              series: [counts.defaults, counts.defaults],
            },
            {
              key: 'used',
              label: t('layouts.metrics.used.label'),
              value: counts.used,
              hint: t('layouts.metrics.used.hint'),
              series: [counts.used, counts.used],
            },
            {
              key: 'free',
              label: t('layouts.metrics.free.label'),
              value: counts.free,
              hint: t('layouts.metrics.free.hint'),
              series: [counts.free, counts.free],
            },
          ]}
        />

        <Section
          title={t('layouts.section.title')}
          description={t('layouts.section.description')}
          action={<ImportButton label={t('actions.import')} onImport={onImport} />}
        >
          <div className='mb-3 flex'>
            <FilterBar fields={filterFields} listing={listing} />
          </div>

          <DataView
            rows={rows}
            columns={columns}
            card={{
              avatar: ({ layout }) => (
                <IconTile tone={layout.is_default ? 'violet' : 'info'}>
                  <LayoutTemplate className='size-4' strokeWidth={1.75} />
                </IconTile>
              ),
              title: ({ layout }) => layout.name,
              subtitle: ({ layout }) => t('layouts.row.identifier', { id: layout.id }),
              badges: ({ layout }) => (
                <>
                  {layout.is_default && defaultPill}
                  {usagePill(layout)}
                </>
              ),
              footer: ({ layout }) => <span>{formatDate(layout.updated_at)}</span>,
            }}
            getKey={({ layout }) => layout.id}
            view={LAYOUTS_VIEW}
            loading={isLoading}
            sort={listing.state.sort}
            onSortChange={listing.setSort}
            emptyLabel={narrowed ? t('common:data_view.no_match') : t('layouts.empty.label')}
            emptyHint={
              narrowed ? t('common:data_view.no_match_server_hint') : t('layouts.empty.hint')
            }
            emptyAction={
              narrowed ? (
                <Button variant='outline' onClick={listing.clearFilters}>
                  {t('common:data_view.clear_filter')}
                </Button>
              ) : (
                createButton
              )
            }
          />

          {pagination && (
            <div className='mt-3 px-1'>
              <PaginationBar pagination={pagination} onPageChange={listing.setPage} />
            </div>
          )}
        </Section>
      </div>
    </PageShell>
  )
}
