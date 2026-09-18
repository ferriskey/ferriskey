import { Trans, useTranslation } from 'react-i18next'
import { AlertTriangle, ArrowLeft, Check, Palette } from 'lucide-react'
import { Button } from '@/components/kit/button'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
import SaveBar from '@/components/kit/save-bar'
import { DangerZone } from '@/components/kit/danger-zone'
import { DetailHeader, IconTile, PageShell, PageTabs, Pill, type TabItem } from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import type { Schemas } from '@/api/api.client'
import { PORTAL_PAGES, humanizeBlockType, labelForPortalPage } from '../portal-pages'
import type { PortalPageStatus } from '../theme-validation'
import ThemeTokensTab from './theme-tokens-tab'
import ThemeLayoutTab from './theme-layout-tab'
import ThemePagesTab from './theme-pages-tab'
import { formatDate } from '@/utils/format-date'

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
  const { t } = useTranslation('portal')

  const backButton = (
    <Button variant='ghost' size='sm' className='-ml-2 mb-2 text-neutral-500 dark:text-neutral-400' onClick={onBack}>
      <ArrowLeft className='size-3.5' />
      {t('header.title')}
    </Button>
  )

  if (isLoading) {
    return (
      <PageShell>
        <div className='h-4 w-24 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
        <div className='mt-4 flex items-center gap-3'>
          <div className='size-15 animate-pulse rounded-md bg-neutral-100 dark:bg-fk-raised' />
          <div className='space-y-2'>
            <div className='h-5 w-48 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
            <div className='h-4 w-32 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
          </div>
        </div>
      </PageShell>
    )
  }

  if (!theme) {
    return (
      <PageShell>
        {backButton}
        <div className={cn(tokens.surface.panel, 'grid place-items-center px-6 py-16')}>
          <p className='text-sm font-medium text-neutral-700 dark:text-neutral-300'>
            {t('detail.not_found.title')}
          </p>
          <p className='mt-1 max-w-sm text-center text-sm text-neutral-500 dark:text-neutral-400'>
            {t('detail.not_found.hint')}
          </p>
        </div>
      </PageShell>
    )
  }

  const failures = statuses.filter((s) => s.missing.length > 0)

  return (
    <PageShell>
      <DetailHeader
        onBack={onBack}
        backLabel={t('header.title')}
        icon={
          <IconTile tone={isActive ? 'primary' : 'info'} className='size-15'>
            <Palette className='size-6' strokeWidth={1.5} />
          </IconTile>
        }
        title={name}
        pills={
          <>
            {isActive ? (
              <Pill tone='success'>
                <Check className='size-3' strokeWidth={3} />
                {t('themes.row.active')}
              </Pill>
            ) : (
              <Pill tone='neutral'>{t('detail.draft')}</Pill>
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
          </>
        }
        meta={
          <div className='flex shrink-0 items-center gap-3'>
            {!isActive &&
              (failures.length === 0 ? (
                <Button variant='outline' onClick={onActivate} disabled={isActivating}>
                  {isActivating ? t('detail.activating') : t('detail.activate')}
                </Button>
              ) : (
                <Tooltip>
                  <TooltipTrigger asChild>
                    <span className='inline-flex items-center gap-1.5 text-xs text-fk-amber'>
                      <AlertTriangle className='size-3.5' strokeWidth={2} />
                      {t('detail.activation_blocked')}
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

            <dl className='text-right text-xs text-neutral-500 dark:text-neutral-400'>
              <dt className='sr-only'>{t('detail.meta.updated_label')}</dt>
              <dd className='tnum'>
                {t('detail.meta.updated', { date: formatDate(theme.updated_at) })}
              </dd>
              <dt className='sr-only'>{t('detail.meta.identifier_label')}</dt>
              <dd className='font-mono-ui text-[11px] text-neutral-400 dark:text-neutral-500'>{theme.id}</dd>
            </dl>
          </div>
        }
      />

      <div
        className={cn(
          'mt-4 rounded-sm border px-4 py-3 text-xs',
          isActive
            ? 'border-fk-info-border bg-fk-info-soft/40 text-neutral-700 dark:text-neutral-300'
            : 'border-fk-line bg-neutral-50 text-neutral-600 dark:bg-fk-surface dark:text-neutral-400'
        )}
      >
        {isActive ? t('detail.banner.active') : t('detail.banner.draft')}
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
                <div className={cn(tokens.surface.panel, 'px-4 py-3 text-xs text-neutral-500 dark:text-neutral-400')}>
                  {t('themes.row.delete_blocked')}
                </div>
              ) : (
                <DangerZone
                  resourceName={theme.name}
                  label={t('detail.danger.label')}
                  description={t('detail.danger.description', { total: PORTAL_PAGES.length })}
                  buttonLabel={t('detail.danger.button')}
                  confirmTitle={t('detail.danger.confirm_title')}
                  confirmDescription={t('detail.danger.confirm_description', {
                    name: theme.name,
                    total: PORTAL_PAGES.length,
                  })}
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
        title={t('detail.save_bar.title', { count: dirtyCount })}
        description={t('detail.save_bar.description')}
        onCancel={onDiscard}
        cancelLabel={t('detail.save_bar.discard')}
        actions={[
          {
            label: isSaving ? t('detail.save_bar.saving') : t('detail.save_bar.save'),
            onClick: onSave,
          },
        ]}
      />
    </PageShell>
  )
}
