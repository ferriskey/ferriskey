import { AlertTriangle, ArrowLeft, Check, Palette } from 'lucide-react'
import { Button } from '@/components/kit/button'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
import SaveBar from '@/components/kit/save-bar'
import { DangerZone } from '@/components/kit/danger-zone'
import { IconTile, PageTabs, Pill, type TabItem } from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import type { Schemas } from '@/api/api.client'
import { PORTAL_PAGES, humanizeBlockType, labelForPortalPage } from '../portal-pages'
import type { PortalPageStatus } from '../theme-validation'
import ThemeTokensTab from './theme-tokens-tab'
import ThemeLayoutTab from './theme-layout-tab'
import ThemePagesTab from './theme-pages-tab'
import { formatDate } from '@/next/shared/format-date'

export interface PagePortalThemeDetailProps {
  theme?: Schemas.PortalTheme
  layouts: Schemas.PortalLayout[]
  isLoading: boolean
  isActive: boolean
  isActivating: boolean
  isSaving: boolean
  tab: string
  tabs: TabItem[]
  name: string
  nameError?: string
  layoutId: string
  savedLayoutId: string
  statuses: PortalPageStatus[]
  dirtyCount: number
  pageHref: (pageType: string) => string
  onNameChange: (value: string) => void
  onLayoutChange: (value: string) => void
  onActivate: () => void
  onBack: () => void
  onDiscard: () => void
  onSave: () => void
  onDelete: () => void
}

export default function PagePortalThemeDetail({
  theme,
  layouts,
  isLoading,
  isActive,
  isActivating,
  isSaving,
  tab,
  tabs,
  name,
  nameError,
  layoutId,
  savedLayoutId,
  statuses,
  dirtyCount,
  pageHref,
  onNameChange,
  onLayoutChange,
  onActivate,
  onBack,
  onDiscard,
  onSave,
  onDelete,
}: PagePortalThemeDetailProps) {
  const container = cn('mx-auto', tokens.page.maxWidth, tokens.page.padding)
  const backButton = (
    <Button variant='ghost' size='sm' className='-ml-2 mb-2 text-neutral-500' onClick={onBack}>
      <ArrowLeft className='size-3.5' />
      Portal
    </Button>
  )

  if (isLoading) {
    return (
      <div className={container}>
        <div className='h-4 w-24 animate-pulse rounded bg-neutral-100' />
        <div className='mt-4 flex items-center gap-3'>
          <div className='size-15 animate-pulse rounded-md bg-neutral-100' />
          <div className='space-y-2'>
            <div className='h-5 w-48 animate-pulse rounded bg-neutral-100' />
            <div className='h-4 w-32 animate-pulse rounded bg-neutral-100' />
          </div>
        </div>
      </div>
    )
  }

  if (!theme) {
    return (
      <div className={container}>
        {backButton}
        <div className={cn(tokens.surface.panel, 'grid place-items-center px-6 py-16')}>
          <p className='text-sm font-medium text-neutral-700'>Theme not found</p>
          <p className='mt-1 max-w-sm text-center text-sm text-neutral-500'>
            It may have been deleted, or it belongs to another realm.
          </p>
        </div>
      </div>
    )
  }

  const failures = statuses.filter((s) => s.missing.length > 0)

  return (
    <div className={container}>
      {backButton}

      <div className='flex flex-wrap items-start justify-between gap-4'>
        <div className='flex items-center gap-3'>
          <IconTile tone={isActive ? 'primary' : 'info'} className='size-15'>
            <Palette className='size-6' strokeWidth={1.5} />
          </IconTile>
          <div className='min-w-0'>
            <h1 className={tokens.header.title}>{name}</h1>
            <div className='mt-1.5 flex flex-wrap items-center gap-2'>
              {isActive ? (
                <Pill tone='success'>
                  <Check className='size-3' strokeWidth={3} />
                  active
                </Pill>
              ) : (
                <Pill tone='neutral'>draft</Pill>
              )}
              <Pill tone={failures.length === 0 ? 'neutral' : 'amber'}>
                <span className='tnum'>
                  {PORTAL_PAGES.length - failures.length}/{PORTAL_PAGES.length}
                </span>{' '}
                pages valid
              </Pill>
            </div>
          </div>
        </div>

        <div className='flex shrink-0 items-center gap-3'>
          {!isActive &&
            (failures.length === 0 ? (
              <Button variant='outline' onClick={onActivate} disabled={isActivating}>
                {isActivating ? 'Activating…' : 'Activate this theme'}
              </Button>
            ) : (
              <Tooltip>
                <TooltipTrigger asChild>
                  <span className='inline-flex items-center gap-1.5 text-xs text-fk-amber'>
                    <AlertTriangle className='size-3.5' strokeWidth={2} />
                    Activation blocked
                  </span>
                </TooltipTrigger>
                <TooltipContent side='left' className='max-w-xs'>
                  {failures
                    .map(
                      (f) =>
                        `${labelForPortalPage(f.pageType)} (${f.missing.map(humanizeBlockType).join(', ')})`
                    )
                    .join(' · ')}
                </TooltipContent>
              </Tooltip>
            ))}

          <dl className='text-right text-xs text-neutral-500'>
            <dt className='sr-only'>Updated at</dt>
            <dd className='tnum'>
              Updated {formatDate(theme.updated_at)}
            </dd>
            <dt className='sr-only'>Identifier</dt>
            <dd className='font-mono-ui text-[11px] text-neutral-400'>{theme.id}</dd>
          </dl>
        </div>
      </div>

      <div
        className={cn(
          'mt-4 rounded-sm border px-4 py-3 text-xs',
          isActive
            ? 'border-fk-info-border bg-fk-info-soft/40 text-neutral-700'
            : 'border-fk-line bg-neutral-50 text-neutral-600'
        )}
      >
        {isActive
          ? 'The portal renders this theme: every page you save is validated immediately, and a missing required block is refused.'
          : 'Draft: pages save without constraint. Required blocks are checked when the theme is activated.'}
      </div>

      <PageTabs tabs={tabs} value={tab} className='mt-4'>
        <div className={tokens.page.blockGap}>
          {tab === 'theme' && (
            <>
              <ThemeTokensTab
                name={name}
                onNameChange={onNameChange}
                nameError={nameError}
              />
              {isActive ? (
                <div className={cn(tokens.surface.panel, 'px-4 py-3 text-xs text-neutral-500')}>
                  The active theme cannot be deleted. Activate another one first.
                </div>
              ) : (
                <DangerZone
        resourceName={theme.name}
                  label='Delete this theme'
                  description='Its design tokens and the trees of its twelve pages are lost for good.'
                  buttonLabel='Delete theme'
                  confirmTitle='Delete theme'
                  confirmDescription={`This will permanently delete the theme "${theme.name}" and the composition of its twelve pages.`}
                  onConfirm={onDelete}
                />
              )}
            </>
          )}

          {tab === 'layout' && (
            <ThemeLayoutTab
              layouts={layouts}
              layoutId={layoutId}
              savedLayoutId={savedLayoutId}
              onLayoutChange={onLayoutChange}
            />
          )}

          {tab === 'pages' && <ThemePagesTab statuses={statuses} pageHref={pageHref} />}
        </div>
      </PageTabs>

      <SaveBar
        show={dirtyCount > 0}
        title={`${dirtyCount} unsaved change${dirtyCount > 1 ? 's' : ''}`}
        description='The portal only reads what has been saved.'
        onCancel={onDiscard}
        cancelLabel='Discard'
        actions={[{ label: isSaving ? 'Saving…' : 'Save changes', onClick: onSave }]}
      />
    </div>
  )
}
