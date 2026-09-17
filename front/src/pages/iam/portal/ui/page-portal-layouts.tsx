import { Trans, useTranslation } from 'react-i18next'
import { Download, LayoutTemplate, Plus, Trash2 } from 'lucide-react'
import { Button } from '@/components/kit/button'
import { Skeleton } from '@/components/ui/skeleton'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
import { IconTile, MetricsBand, PageShell, Pill, Section } from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import type { Schemas } from '@/api/api.client'
import { PORTAL_TAB_LAYOUTS, PortalPageHeader } from './portal-page-header'
import { ImportButton } from './import-button'
import { formatDate } from '@/utils/format-date'

export interface PortalLayoutRow {
  layout: Schemas.PortalLayout
  nodes: number
  usedBy: string[]
}

export interface PagePortalLayoutsProps {
  rows: PortalLayoutRow[]
  isLoading: boolean
  onCreate: () => void
  onEdit: (layoutId: string) => void
  onDelete: (layoutId: string) => void
  onExport: (layoutId: string) => void
  onImport: (file: File) => void
}


function refusalKeyFor({ layout, usedBy }: PortalLayoutRow): string | null {
  if (layout.is_default) return 'layouts.row.refusal.default'
  if (usedBy.length > 0) return 'layouts.row.refusal.in_use'
  return null
}

export default function PagePortalLayouts({
  rows,
  isLoading,
  onCreate,
  onEdit,
  onDelete,
  onExport,
  onImport,
}: PagePortalLayoutsProps) {
  const { t } = useTranslation('portal')

  const createButton = (
    <Button onClick={onCreate}>
      <Plus /> {t('layouts.create')}
    </Button>
  )

  const used = rows.filter((r) => r.usedBy.length > 0).length

  return (
    <PageShell>
      <PortalPageHeader tab={PORTAL_TAB_LAYOUTS} actions={createButton} />

      <div className={cn('mt-4', tokens.page.blockGap)}>
        <MetricsBand
          metrics={[
            {
              key: 'total',
              label: t('layouts.metrics.total.label'),
              value: rows.length,
              hint: t('layouts.metrics.total.hint'),
            },
            {
              key: 'default',
              label: t('layouts.metrics.default.label'),
              value: rows.filter((r) => r.layout.is_default).length,
              hint: t('layouts.metrics.default.hint'),
            },
            {
              key: 'used',
              label: t('layouts.metrics.used.label'),
              value: used,
              hint: t('layouts.metrics.used.hint'),
            },
            {
              key: 'free',
              label: t('layouts.metrics.free.label'),
              value: rows.filter((r) => refusalKeyFor(r) === null).length,
              hint: t('layouts.metrics.free.hint'),
            },
          ]}
        />

        <Section
          title={t('layouts.section.title')}
          description={t('layouts.section.description')}
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
              <LayoutTemplate className='size-8 text-neutral-300 dark:text-neutral-600' strokeWidth={1.5} />
              <p className='max-w-sm text-center text-sm text-neutral-500 dark:text-neutral-400'>
                {t('layouts.empty')}
              </p>
              {createButton}
            </div>
          ) : (
            <ul className={tokens.surface.divider}>
              {rows.map((row) => {
                const { layout, nodes, usedBy } = row
                const refusalKey = refusalKeyFor(row)
                return (
                  <li key={layout.id} className='flex items-center gap-3 py-3'>
                    <IconTile tone={layout.is_default ? 'violet' : 'info'}>
                      <LayoutTemplate className='size-4' strokeWidth={1.75} />
                    </IconTile>

                    <div className='min-w-0 flex-1'>
                      <div className='flex flex-wrap items-center gap-2'>
                        <button
                          type='button'
                          onClick={() => onEdit(layout.id)}
                          className='cursor-pointer text-[13px] font-medium text-neutral-900 dark:text-neutral-100 hover:underline'
                        >
                          {layout.name}
                        </button>
                        {layout.is_default && <Pill tone='violet'>{t('layouts.row.default')}</Pill>}
                        <Pill tone={usedBy.length > 0 ? 'info' : 'neutral'}>
                          {usedBy.length > 0
                            ? t('layouts.row.theme_count', { count: usedBy.length })
                            : t('layouts.row.unused')}
                        </Pill>
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
                          {t(refusalKey, {
                            themes: usedBy.join(', '),
                            count: usedBy.length,
                          })}
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
                  </li>
                )
              })}
            </ul>
          )}
        </Section>
      </div>
    </PageShell>
  )
}
