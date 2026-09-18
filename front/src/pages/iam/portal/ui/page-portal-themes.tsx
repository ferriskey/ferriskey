import { useState } from 'react'
import { Link } from 'react-router-dom'
import { Trans, useTranslation } from 'react-i18next'
import { AlertTriangle, Check, Download, Palette, Plus, Trash2 } from 'lucide-react'
import { Button } from '@/components/kit/button'
import { Input } from '@/components/ui/input'
import { Skeleton } from '@/components/ui/skeleton'
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
import { IconTile, MetricsBand, PageShell, Pill, Section } from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import type { Schemas } from '@/api/api.client'
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

export interface PagePortalThemesProps {
  rows: PortalThemeRow[]
  isLoading: boolean
  isCreating: boolean
  themeHref: (themeId: string) => string
  onCreate: (name: string) => void
  onActivate: (themeId: string) => void
  onDelete: (themeId: string) => void
  onExport: (themeId: string) => void
  onImport: (file: File) => void
}

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

  const activatable = rows.filter((r) => r.failures.length === 0).length
  const activeCount = rows.filter((r) => r.isActive).length

  const createButton = (
    <Button onClick={() => setCreateOpen(true)}>
      <Plus /> {t('themes.create')}
    </Button>
  )

  return (
    <PageShell>
      <PortalPageHeader tab={PORTAL_TAB_THEMES} actions={createButton} />

      <div className={cn('mt-4', tokens.page.blockGap)}>
        <MetricsBand
          metrics={[
            {
              key: 'total',
              label: t('themes.metrics.total.label'),
              value: rows.length,
              hint: t('themes.metrics.total.hint'),
            },
            {
              key: 'active',
              label: t('themes.metrics.active.label'),
              value: activeCount,
              hint:
                activeCount > 0
                  ? t('themes.metrics.active.hint')
                  : t('themes.metrics.active.empty_hint'),
            },
            {
              key: 'activatable',
              label: t('themes.metrics.activatable.label'),
              value: activatable,
              hint: t('themes.metrics.activatable.hint'),
            },
            {
              key: 'pages',
              label: t('themes.metrics.pages.label'),
              value: PORTAL_PAGES.length,
              hint: t('themes.metrics.pages.hint'),
            },
          ]}
        />

        <Section
          title={t('themes.section.title')}
          description={t('themes.section.description')}
          action={<ImportButton label={t('actions.import')} onImport={onImport} />}
        >
          {isLoading ? (
            <ul className={tokens.surface.divider}>
              {Array.from({ length: 3 }).map((_, i) => (
                <li key={i} className='flex items-center gap-3 py-3'>
                  <Skeleton className='size-9 rounded-md' />
                  <div className='flex-1 space-y-2'>
                    <Skeleton className='h-4 w-40' />
                    <Skeleton className='h-3 w-56' />
                  </div>
                  <Skeleton className='h-6 w-20 rounded-md' />
                </li>
              ))}
            </ul>
          ) : rows.length === 0 ? (
            <div className='grid place-items-center gap-3 py-16'>
              <Palette className='size-8 text-neutral-300 dark:text-neutral-600' strokeWidth={1.5} />
              <p className='max-w-sm text-center text-sm text-neutral-500 dark:text-neutral-400'>
                {t('themes.empty', { total: PORTAL_PAGES.length })}
              </p>
              {createButton}
            </div>
          ) : (
            <ul className={tokens.surface.divider}>
              {rows.map(({ theme, isActive, layoutName, failures }) => (
                <li key={theme.id} className='flex items-center gap-3 py-3'>
                  <IconTile tone={isActive ? 'primary' : 'info'}>
                    <Palette className='size-4' strokeWidth={1.75} />
                  </IconTile>

                  <div className='min-w-0 flex-1'>
                    <div className='flex flex-wrap items-center gap-2'>
                      <Link
                        to={themeHref(theme.id)}
                        className='text-[13px] font-medium text-neutral-900 dark:text-neutral-100 hover:underline'
                      >
                        {theme.name}
                      </Link>
                      {isActive && (
                        <Pill tone='success'>
                          <Check className='size-3' strokeWidth={3} />
                          {t('themes.row.active')}
                        </Pill>
                      )}
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
                    </div>
                    <p className='mt-0.5 truncate text-xs text-neutral-500 dark:text-neutral-400'>
                      {t('themes.row.summary', {
                        layout: layoutName
                          ? t('themes.row.layout', { name: layoutName })
                          : t('themes.row.no_layout'),
                        date: formatDate(theme.updated_at),
                      })}
                    </p>
                    <p className='font-mono-ui mt-0.5 truncate text-[11px] text-neutral-400 dark:text-neutral-500'>
                      {t('themes.row.identifier', { id: theme.id })}
                    </p>
                  </div>

                  <Swatches config={theme.config} />

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
                </li>
              ))}
            </ul>
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
