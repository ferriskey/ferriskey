import { useState } from 'react'
import { Link } from 'react-router-dom'
import { AlertTriangle, Check, Download, Palette, Plus, Trash2 } from 'lucide-react'
import { Button } from '@/components/ui/button'
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
import { IconTile, MetricsBand, Pill, Section } from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import type { Schemas } from '@/api/api.client'
import { PORTAL_PAGES, labelForPortalPage, humanizeBlockType } from '../portal-pages'
import type { PortalPageStatus } from '../theme-validation'
import { PortalPageHeader } from './portal-page-header'
import { ImportButton } from './import-button'

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

const formatDate = (iso: string) => new Date(iso).toLocaleDateString('en-GB')

function Swatches({ config }: { config: Schemas.PortalThemeConfig }) {
  const colors = (config?.colors ?? {}) as Record<string, string | undefined>
  return (
    <span className='flex shrink-0 items-center gap-1'>
      {SWATCH_KEYS.map((key) => (
        <span
          key={key}
          title={`${key} · ${colors[key] ?? 'default'}`}
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
      <Plus /> New theme
    </Button>
  )

  return (
    <div className={cn('mx-auto', tokens.page.maxWidth, tokens.page.padding)}>
      <PortalPageHeader tab='themes' actions={createButton} />

      <div className={cn('mt-4', tokens.page.blockGap)}>
        <MetricsBand
          metrics={[
            { key: 'total', label: 'Themes', value: rows.length, hint: 'in this realm' },
            {
              key: 'active',
              label: 'Active',
              value: activeCount,
              hint: activeCount > 0 ? 'rendered by the portal' : 'the portal falls back',
            },
            {
              key: 'activatable',
              label: 'Activatable',
              value: activatable,
              hint: 'every page valid',
            },
            {
              key: 'pages',
              label: 'Pages per theme',
              value: PORTAL_PAGES.length,
              hint: 'page types',
            },
          ]}
        />

        <Section
          title='Themes'
          description='Only one theme is active at a time; it is the one the portal renders.'
          action={<ImportButton label='Import' onImport={onImport} />}
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
              <Palette className='size-8 text-neutral-300' strokeWidth={1.5} />
              <p className='max-w-sm text-center text-sm text-neutral-500'>
                No theme yet. A theme carries the colours, the typography and the twelve
                pages the portal renders.
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
                        className='text-[13px] font-medium text-neutral-900 hover:underline'
                      >
                        {theme.name}
                      </Link>
                      {isActive && (
                        <Pill tone='success'>
                          <Check className='size-3' strokeWidth={3} />
                          active
                        </Pill>
                      )}
                      <Pill tone={failures.length === 0 ? 'neutral' : 'amber'}>
                        <span className='tnum'>
                          {PORTAL_PAGES.length - failures.length}/{PORTAL_PAGES.length}
                        </span>{' '}
                        pages valid
                      </Pill>
                    </div>
                    <p className='mt-0.5 truncate text-xs text-neutral-500'>
                      {layoutName ? `Layout ${layoutName}` : 'No layout'} · updated{' '}
                      {formatDate(theme.updated_at)}
                    </p>
                    <p className='font-mono-ui mt-0.5 truncate text-[11px] text-neutral-400'>
                      theme_id: {theme.id}
                    </p>
                  </div>

                  <Swatches config={theme.config} />

                  {isActive ? (
                    <span className='px-2 text-xs text-neutral-400'>active theme</span>
                  ) : failures.length > 0 ? (
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <span className='inline-flex items-center gap-1.5 px-2 text-xs text-fk-amber'>
                          <AlertTriangle className='size-3.5' strokeWidth={2} />
                          <span className='tnum'>{failures.length}</span> incomplete page
                          {failures.length > 1 ? 's' : ''}
                        </span>
                      </TooltipTrigger>
                      <TooltipContent side='left' className='max-w-xs'>
                        Cannot be activated: {describeFailures(failures)}
                      </TooltipContent>
                    </Tooltip>
                  ) : (
                    <Button variant='outline' size='sm' onClick={() => onActivate(theme.id)}>
                      Activate
                    </Button>
                  )}

                  <Button
                    variant='ghost'
                    size='icon'
                    className='size-8 text-neutral-400'
                    aria-label={`Export ${theme.name}`}
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
                        The active theme cannot be deleted. Activate another one first.
                      </TooltipContent>
                    </Tooltip>
                  ) : (
                    <Button
                      variant='ghost'
                      size='icon'
                      aria-label={`Delete ${theme.name}`}
                      className='size-8 text-neutral-400 hover:text-fk-danger'
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
            <DialogTitle>New portal theme</DialogTitle>
          </DialogHeader>
          <p className='text-sm text-neutral-500'>
            The twelve pages are pre-filled with their default composition, so the theme is
            activatable as soon as it is created.
          </p>
          <Input
            placeholder='Theme name'
            value={newName}
            onChange={(e) => setNewName(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter') submitCreate()
            }}
          />
          <DialogFooter>
            <Button variant='outline' onClick={() => setCreateOpen(false)}>
              Cancel
            </Button>
            <Button onClick={submitCreate} disabled={isCreating || !newName.trim()}>
              {isCreating ? 'Creating…' : 'Create'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
