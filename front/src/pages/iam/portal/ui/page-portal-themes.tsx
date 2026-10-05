import { useState } from 'react'
import { Link } from 'react-router-dom'
import { Trans, useTranslation } from 'react-i18next'
import { AlertTriangle, Check, Download, Palette, Plus, Trash2 } from 'lucide-react'
import { Button } from '@/components/kit/button'
import { Input } from '@/components/ui/input'
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
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
import { portalLayoutRelationSource } from '@/api/portal-layout.relation'
import { PORTAL_PAGES, labelForPortalPage, humanizeBlockType } from '../portal-pages'
import type { PortalPageStatus } from '../theme-validation'
import { PORTAL_TAB_THEMES, PortalPageHeader } from './portal-page-header'
import { ImportButton } from './import-button'
import { formatDate } from '@/utils/format-date'

export interface PortalThemeRow {
  theme: Schemas.PortalTheme
  isActive: boolean
  layoutName?: string
  failures: PortalPageStatus[]
}

export interface PortalThemeCounts {
  total: number
  active: number
  activatable: number
}

export interface PagePortalThemesProps {
  rows: PortalThemeRow[]
  listing: PagedListing
  pagination: PaginationMetadata | undefined
  counts: PortalThemeCounts
  isLoading: boolean
  isCreating: boolean
  themeHref: (themeId: string) => string
  onCreate: (name: string) => void
  onActivate: (themeId: string) => void
  onDelete: (themeId: string) => void
  onExport: (themeId: string) => void
  onImport: (file: File) => void
}

const THEMES_VIEW: ViewMode = 'list'

const SWATCH_KEYS = [
  'primaryButton',
  'pageBackground',
  'widgetBackground',
  'links',
  'error',
] as const


function Swatches({ config }: { config: Schemas.PortalThemeConfig }) {
  const { t } = useTranslation('portal')
  const colors = (config?.colors ?? {}) as Record<string, string | undefined>
  return (
    <span className='flex shrink-0 items-center gap-1'>
      {SWATCH_KEYS.map((key) => (
        <span
          key={key}
          title={t('themes.row.swatch', {
            token: key,
            value: colors[key] ?? t('themes.row.swatch_default'),
          })}
          className='size-3.5 rounded-sm border border-fk-line'
          style={{ backgroundColor: colors[key] ?? 'transparent' }}
        />
      ))}
    </span>
  )
}

function describeFailures(failures: PortalPageStatus[]) {
  return failures
    .map(
      (f) =>
        `${labelForPortalPage(f.pageType)} (${f.missing.map(humanizeBlockType).join(', ')})`
    )
    .join(' · ')
}

export default function PagePortalThemes({
  rows,
  listing,
  pagination,
  counts,
  isLoading,
  isCreating,
  themeHref,
  onCreate,
  onActivate,
  onDelete,
  onExport,
  onImport,
}: PagePortalThemesProps) {
  const { t } = useTranslation('portal')
  const [createOpen, setCreateOpen] = useState(false)
  const [newName, setNewName] = useState('')

  const submitCreate = () => {
    if (!newName.trim()) return
    onCreate(newName.trim())
    setNewName('')
    setCreateOpen(false)
  }

  const narrowed = Object.values(listing.state.filters).some(Boolean)

  const createButton = (
    <Button onClick={() => setCreateOpen(true)}>
      <Plus /> {t('themes.create')}
    </Button>
  )

  const filterFields: FilterField[] = [
    { kind: 'text', key: 'name', label: t('themes.filters.name') },
    {
      kind: 'relation',
      key: 'layout_id',
      label: t('themes.filters.layout'),
      relation: portalLayoutRelationSource,
    },
    { kind: 'boolean', key: 'activatable', label: t('themes.filters.activatable') },
  ]

  const pagesPill = (failures: PortalPageStatus[]) => (
    <Pill tone={failures.length === 0 ? 'neutral' : 'amber'}>
      <Trans
        i18nKey='portal:themes.row.pages_valid'
        values={{
          valid: PORTAL_PAGES.length - failures.length,
          total: PORTAL_PAGES.length,
        }}
        components={{ num: <span className='tnum' /> }}
      />
    </Pill>
  )

  const activePill = (
    <Pill tone='success'>
      <Check className='size-3' strokeWidth={3} />
      {t('themes.row.active')}
    </Pill>
  )

  const columns: Column<PortalThemeRow>[] = [
    {
      key: 'name',
      header: t('themes.columns.name'),
      sortKey: 'name',
      render: ({ theme, isActive, failures }) => (
        <div className='flex min-w-0 items-center gap-3'>
          <IconTile tone={isActive ? 'primary' : 'info'}>
            <Palette className='size-4' strokeWidth={1.75} />
          </IconTile>
          <div className='min-w-0'>
            <div className='flex flex-wrap items-center gap-2'>
              <Link
                to={themeHref(theme.id)}
                className='text-[13px] font-medium text-neutral-900 dark:text-neutral-100 hover:underline'
              >
                {theme.name}
              </Link>
              {isActive && activePill}
              {pagesPill(failures)}
            </div>
            <p className='font-mono-ui mt-0.5 truncate text-[11px] text-neutral-400 dark:text-neutral-500'>
              {t('themes.row.identifier', { id: theme.id })}
            </p>
          </div>
        </div>
      ),
    },
    {
      key: 'layout',
      header: t('themes.columns.layout'),
      render: ({ layoutName }) => (
        <span className='text-xs text-neutral-500 dark:text-neutral-400'>
          {layoutName ?? t('themes.row.no_layout')}
        </span>
      ),
    },
    {
      key: 'colors',
      header: t('themes.columns.colors'),
      render: ({ theme }) => <Swatches config={theme.config} />,
    },
    {
      key: 'updated_at',
      header: t('themes.columns.updated_at'),
      sortKey: 'updated_at',
      render: ({ theme }) => (
        <span className='tnum text-xs text-neutral-500 dark:text-neutral-400'>
          {formatDate(theme.updated_at)}
        </span>
      ),
    },
    {
      key: 'created_at',
      header: t('themes.columns.created_at'),
      sortKey: 'created_at',
      render: ({ theme }) => (
        <span className='tnum text-xs text-neutral-500 dark:text-neutral-400'>
          {formatDate(theme.created_at)}
        </span>
      ),
    },
    {
      key: 'actions',
      header: '',
      align: 'right',
      render: ({ theme, isActive, failures }) => (
        <div className='flex items-center justify-end gap-1'>
          {isActive ? (
            <span className='px-2 text-xs text-neutral-400 dark:text-neutral-500'>
              {t('themes.row.active_theme')}
            </span>
          ) : failures.length > 0 ? (
            <Tooltip>
              <TooltipTrigger asChild>
                <span className='inline-flex items-center gap-1.5 px-2 text-xs text-fk-amber'>
                  <AlertTriangle className='size-3.5' strokeWidth={2} />
                  <Trans
                    i18nKey='portal:themes.row.incomplete_pages'
                    count={failures.length}
                    components={{ num: <span className='tnum' /> }}
                  />
                </span>
              </TooltipTrigger>
              <TooltipContent side='left' className='max-w-xs'>
                {t('themes.row.activation_blocked', {
                  detail: describeFailures(failures),
                })}
              </TooltipContent>
            </Tooltip>
          ) : (
            <Button variant='outline' size='sm' onClick={() => onActivate(theme.id)}>
              {t('themes.row.activate')}
            </Button>
          )}

          <Button
            variant='ghost'
            size='icon'
            className='size-8 text-neutral-400 dark:text-neutral-500'
            aria-label={t('themes.row.export', { name: theme.name })}
            onClick={() => onExport(theme.id)}
          >
            <Download className='size-4' />
          </Button>

          {isActive ? (
            <Tooltip>
              <TooltipTrigger asChild>
                <span className='grid size-8 place-items-center text-neutral-200'>
                  <Trash2 className='size-4' />
                </span>
              </TooltipTrigger>
              <TooltipContent side='left' className='max-w-xs'>
                {t('themes.row.delete_blocked')}
              </TooltipContent>
            </Tooltip>
          ) : (
            <Button
              variant='ghost'
              size='icon'
              aria-label={t('themes.row.delete', { name: theme.name })}
              className='size-8 text-neutral-400 dark:text-neutral-500 hover:text-fk-danger'
              onClick={() => onDelete(theme.id)}
            >
              <Trash2 className='size-4' />
            </Button>
          )}
        </div>
      ),
    },
  ]

  return (
    <PageShell>
      <PortalPageHeader tab={PORTAL_TAB_THEMES} actions={createButton} />

      <div className={cn('mt-4', tokens.page.blockGap)}>
        <MetricsBand
          metrics={[
            {
              key: 'total',
              label: t('themes.metrics.total.label'),
              value: counts.total,
              hint: t('themes.metrics.total.hint'),
              series: [counts.total, counts.total],
            },
            {
              key: 'active',
              label: t('themes.metrics.active.label'),
              value: counts.active,
              hint:
                counts.active > 0
                  ? t('themes.metrics.active.hint')
                  : t('themes.metrics.active.empty_hint'),
              series: [counts.active, counts.active],
            },
            {
              key: 'activatable',
              label: t('themes.metrics.activatable.label'),
              value: counts.activatable,
              hint: t('themes.metrics.activatable.hint'),
              series: [counts.activatable, counts.activatable],
            },
            {
              key: 'pages',
              label: t('themes.metrics.pages.label'),
              value: PORTAL_PAGES.length,
              hint: t('themes.metrics.pages.hint'),
              series: [PORTAL_PAGES.length, PORTAL_PAGES.length],
            },
          ]}
        />

        <Section
          title={t('themes.section.title')}
          description={t('themes.section.description')}
          action={<ImportButton label={t('actions.import')} onImport={onImport} />}
        >
          <div className='mb-3 flex'>
            <FilterBar fields={filterFields} listing={listing} />
          </div>

          <DataView
            rows={rows}
            columns={columns}
            card={{
              avatar: ({ isActive }) => (
                <IconTile tone={isActive ? 'primary' : 'info'}>
                  <Palette className='size-4' strokeWidth={1.75} />
                </IconTile>
              ),
              title: ({ theme }) => theme.name,
              subtitle: ({ theme }) => t('themes.row.identifier', { id: theme.id }),
              badges: ({ isActive, failures }) => (
                <>
                  {isActive && activePill}
                  {pagesPill(failures)}
                </>
              ),
              footer: ({ theme, layoutName }) => (
                <>
                  <span>{layoutName ?? t('themes.row.no_layout')}</span>
                  <span>{formatDate(theme.updated_at)}</span>
                </>
              ),
            }}
            getKey={({ theme }) => theme.id}
            view={THEMES_VIEW}
            loading={isLoading}
            sort={listing.state.sort}
            onSortChange={listing.setSort}
            emptyLabel={narrowed ? t('common:data_view.no_match') : t('themes.empty.label')}
            emptyHint={
              narrowed
                ? t('common:data_view.no_match_server_hint')
                : t('themes.empty.hint', { total: PORTAL_PAGES.length })
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

      <Dialog open={createOpen} onOpenChange={setCreateOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t('themes.create_dialog.title')}</DialogTitle>
          </DialogHeader>
          <p className='text-sm text-neutral-500 dark:text-neutral-400'>
            {t('themes.create_dialog.description', { total: PORTAL_PAGES.length })}
          </p>
          <Input
            placeholder={t('themes.create_dialog.name_placeholder')}
            value={newName}
            onChange={(e) => setNewName(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter') submitCreate()
            }}
          />
          <DialogFooter>
            <Button variant='outline' onClick={() => setCreateOpen(false)}>
              {t('themes.create_dialog.cancel')}
            </Button>
            <Button onClick={submitCreate} disabled={isCreating || !newName.trim()}>
              {isCreating
                ? t('themes.create_dialog.submitting')
                : t('themes.create_dialog.submit')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </PageShell>
  )
}
