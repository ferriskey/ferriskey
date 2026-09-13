import { Download, LayoutTemplate, Plus, Trash2 } from 'lucide-react'
import { Button } from '@/components/kit/button'
import { Skeleton } from '@/components/ui/skeleton'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
import { IconTile, MetricsBand, PageShell, Pill, Section } from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import type { Schemas } from '@/api/api.client'
import { PortalPageHeader } from './portal-page-header'
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


function refusalFor({ layout, usedBy }: PortalLayoutRow): string | null {
  if (layout.is_default) {
    return 'The default layout cannot be deleted. Make another layout the default first.'
  }
  if (usedBy.length > 0) {
    return `Used by ${usedBy.join(', ')}. Detach it from ${usedBy.length > 1 ? 'those themes' : 'that theme'} first.`
  }
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
  const createButton = (
    <Button onClick={onCreate}>
      <Plus /> New layout
    </Button>
  )

  const used = rows.filter((r) => r.usedBy.length > 0).length

  return (
    <PageShell>
      <PortalPageHeader tab='layouts' actions={createButton} />

      <div className={cn('mt-4', tokens.page.blockGap)}>
        <MetricsBand
          metrics={[
            { key: 'total', label: 'Layouts', value: rows.length, hint: 'in this realm' },
            {
              key: 'default',
              label: 'Default',
              value: rows.filter((r) => r.layout.is_default).length,
              hint: 'used when a theme names none',
            },
            { key: 'used', label: 'Attached', value: used, hint: 'held by a theme' },
            {
              key: 'free',
              label: 'Deletable',
              value: rows.filter((r) => refusalFor(r) === null).length,
              hint: 'neither default nor held',
            },
          ]}
        />

        <Section
          title='Layouts'
          description='The page structure a theme reuses: where the card, the visual and the footer sit.'
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
              <LayoutTemplate className='size-8 text-neutral-300 dark:text-neutral-600' strokeWidth={1.5} />
              <p className='max-w-sm text-center text-sm text-neutral-500 dark:text-neutral-400'>
                No layout yet. A theme without a layout renders its pages bare, which is a
                valid composition.
              </p>
              {createButton}
            </div>
          ) : (
            <ul className={tokens.surface.divider}>
              {rows.map((row) => {
                const { layout, nodes, usedBy } = row
                const refusal = refusalFor(row)
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
                        {layout.is_default && <Pill tone='violet'>default</Pill>}
                        <Pill tone={usedBy.length > 0 ? 'info' : 'neutral'}>
                          {usedBy.length > 0
                            ? `${usedBy.length} theme${usedBy.length > 1 ? 's' : ''}`
                            : 'unused'}
                        </Pill>
                      </div>
                      <p className='mt-0.5 truncate text-xs text-neutral-500 dark:text-neutral-400'>
                        <span className='tnum'>{nodes}</span> block{nodes === 1 ? '' : 's'} ·
                        updated {formatDate(layout.updated_at)}
                      </p>
                      <p className='font-mono-ui mt-0.5 truncate text-[11px] text-neutral-400 dark:text-neutral-500'>
                        layout_id: {layout.id}
                      </p>
                    </div>

                    <Button variant='outline' size='sm' onClick={() => onEdit(layout.id)}>
                      Open
                    </Button>

                    <Button
                      variant='ghost'
                      size='icon'
                      className='size-8 text-neutral-400 dark:text-neutral-500'
                      aria-label={`Export ${layout.name}`}
                      onClick={() => onExport(layout.id)}
                    >
                      <Download className='size-4' />
                    </Button>

                    {refusal ? (
                      <Tooltip>
                        <TooltipTrigger asChild>
                          <span className='grid size-8 place-items-center text-neutral-200'>
                            <Trash2 className='size-4' />
                          </span>
                        </TooltipTrigger>
                        <TooltipContent side='left' className='max-w-xs'>
                          {refusal}
                        </TooltipContent>
                      </Tooltip>
                    ) : (
                      <Button
                        variant='ghost'
                        size='icon'
                        aria-label={`Delete ${layout.name}`}
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
