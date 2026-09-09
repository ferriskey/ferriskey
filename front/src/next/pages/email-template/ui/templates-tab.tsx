import { useMemo, useState } from 'react'
import { Link } from 'react-router-dom'
import { AlertTriangle, Download, Mail, Pencil, Plus, Search, Trash2 } from 'lucide-react'
import { Button } from '@/components/kit/button'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { ConfirmDeleteAlert } from '@/components/confirm-delete-alert'
import { IconTile, MetricsBand, Pill, Section } from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import { Schemas } from '@/api/api.client'
import { EMAIL_TYPES, emailTypeSpec, formatRelative, type EmailTypeSpec } from '../email-types'

import EmailTemplate = Schemas.EmailTemplate

type Assignments = Record<EmailTypeSpec['assignmentField'], string | null>

export interface TemplatesTabProps {
  templates: EmailTemplate[]
  isLoading: boolean
  assignments: Assignments
  templateHref: (id: string) => string
  onAssign: (field: EmailTypeSpec['assignmentField'], templateId: string | null) => void
  onCreate: () => void
  onEdit: (id: string) => void
  onDelete: (id: string) => void
  onExport: (id: string, format: 'json' | 'mjml') => void
}

const NONE = '__none__'

export default function TemplatesTab({
  templates,
  isLoading,
  assignments,
  templateHref,
  onAssign,
  onCreate,
  onEdit,
  onDelete,
  onExport,
}: TemplatesTabProps) {
  const [query, setQuery] = useState('')
  const [typeFilter, setTypeFilter] = useState<string>('all')
  const [pendingDelete, setPendingDelete] = useState<EmailTemplate | undefined>(undefined)

  const usedBy = (templateId: string) =>
    EMAIL_TYPES.filter((spec) => assignments[spec.assignmentField] === templateId)

  const rows = useMemo(() => {
    const needle = query.trim().toLowerCase()
    const matching = templates.filter((template) => {
      if (typeFilter !== 'all' && template.email_type !== typeFilter) return false
      if (!needle) return true
      return `${template.name} ${template.email_type}`.toLowerCase().includes(needle)
    })
    return [...matching].sort(
      (a, b) =>
        new Date(b.updated_at || b.created_at).getTime() -
        new Date(a.updated_at || a.created_at).getTime()
    )
  }, [templates, query, typeFilter])

  const unassigned = EMAIL_TYPES.filter((spec) => !assignments[spec.assignmentField])
  const pendingUses = pendingDelete ? usedBy(pendingDelete.id) : []

  return (
    <>
      <div className={tokens.page.blockGap}>
        {!isLoading && unassigned.length > 0 && (
          <div className='flex items-start gap-2.5 rounded-sm border border-fk-amber-border bg-fk-amber-soft/50 px-4 py-3'>
            <AlertTriangle className='mt-0.5 size-4 shrink-0 text-fk-amber' strokeWidth={2} />
            <p className='text-xs text-neutral-700 dark:text-neutral-300'>
              {unassigned.map((spec) => spec.label).join(', ')}{' '}
              {unassigned.length > 1 ? 'have no template assigned' : 'has no template assigned'} —
              FerrisKey sends its plain-text default email for{' '}
              {unassigned.length > 1 ? 'those' : 'that one'}.
            </p>
          </div>
        )}

        <MetricsBand
          metrics={[
            {
              key: 'total',
              label: 'Templates',
              value: templates.length,
              hint: 'in this realm',
            },
            ...EMAIL_TYPES.map((spec) => ({
              key: spec.key,
              label: spec.short,
              value: templates.filter((template) => template.email_type === spec.key).length,
              hint: assignments[spec.assignmentField] ? 'assigned' : 'none assigned',
            })),
          ]}
        />

        <Section
          title='Assignment'
          description='One template per email. Without an assignment, FerrisKey sends its plain-text default email.'
        >
          {EMAIL_TYPES.map((spec) => {
            const candidates = templates.filter((template) => template.email_type === spec.key)
            return (
              <div
                key={spec.key}
                className='grid gap-x-8 gap-y-2 py-4 md:grid-cols-[minmax(0,20rem)_minmax(0,1fr)]'
              >
                <div className='flex min-w-0 items-start gap-2.5'>
                  <IconTile tone={spec.tone} className='size-7'>
                    <spec.icon className='size-3.5' strokeWidth={1.75} />
                  </IconTile>
                  <div className='min-w-0'>
                    <p className='text-sm font-medium text-neutral-900 dark:text-neutral-100'>{spec.label}</p>
                    <p className='mt-0.5 text-xs leading-relaxed text-neutral-500 dark:text-neutral-400'>
                      {spec.trigger}
                    </p>
                  </div>
                </div>
                <div className='min-w-0 max-w-sm'>
                  <Select
                    value={assignments[spec.assignmentField] ?? NONE}
                    onValueChange={(value) =>
                      onAssign(spec.assignmentField, value === NONE ? null : value)
                    }
                  >
                    <SelectTrigger className='w-full'>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent position='popper'>
                      <SelectItem value={NONE}>Not configured</SelectItem>
                      {candidates.map((template) => (
                        <SelectItem key={template.id} value={template.id}>
                          {template.name}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                  {candidates.length === 0 && (
                    <p className='mt-1.5 text-xs text-neutral-400 dark:text-neutral-500'>
                      No template of this type in this realm.
                    </p>
                  )}
                </div>
              </div>
            )
          })}
        </Section>

        <Section
          title='Templates'
          description='The type of a template is fixed at creation: it decides the variables available to it.'
          action={
            <div className='flex items-center gap-2'>
              <label className='relative flex h-8 w-52 items-center'>
                <Search className='pointer-events-none absolute left-2.5 size-3.5 text-neutral-400 dark:text-neutral-500' />
                <input
                  type='search'
                  value={query}
                  onChange={(event) => setQuery(event.target.value)}
                  placeholder='Filter templates…'
                  className='h-full w-full rounded-md border border-fk-line pl-8 pr-2 text-[13px] outline-none placeholder:text-neutral-400 focus:border-fk-primary-border'
                />
              </label>
              <div className='flex overflow-hidden rounded-md border border-fk-line'>
                {['all', ...EMAIL_TYPES.map((spec) => spec.key)].map((key) => (
                  <button
                    key={key}
                    type='button'
                    onClick={() => setTypeFilter(key)}
                    aria-pressed={typeFilter === key}
                    className={cn(
                      'px-2 py-1 text-xs transition-colors',
                      typeFilter === key
                        ? 'bg-fk-primary-soft font-medium text-fk-primary-text'
                        : 'text-neutral-500 hover:bg-neutral-50 dark:text-neutral-400 dark:hover:bg-neutral-900'
                    )}
                  >
                    {key === 'all' ? 'All' : emailTypeSpec(key).short}
                  </button>
                ))}
              </div>
            </div>
          }
          contained={isLoading || rows.length > 0}
        >
          {isLoading ? (
            Array.from({ length: 3 }).map((_, index) => (
              <div key={`skeleton-${index}`} className='flex items-center gap-3 py-3'>
                <div className='size-9 animate-pulse rounded-md bg-neutral-100 dark:bg-neutral-800' />
                <div className='flex-1 space-y-2'>
                  <div className='h-3.5 w-48 animate-pulse rounded bg-neutral-100 dark:bg-neutral-800' />
                  <div className='h-3 w-32 animate-pulse rounded bg-neutral-100 dark:bg-neutral-800' />
                </div>
              </div>
            ))
          ) : rows.length > 0 ? (
            rows.map((template) => {
              const spec = emailTypeSpec(template.email_type)
              const uses = usedBy(template.id)
              return (
                <div key={template.id} className='flex items-center gap-3 py-3'>
                  <IconTile tone={spec.tone}>
                    <spec.icon className='size-4' strokeWidth={1.75} />
                  </IconTile>

                  <div className='min-w-0 flex-1'>
                    <div className='flex flex-wrap items-center gap-2'>
                      <Link
                        to={templateHref(template.id)}
                        className='text-[13px] font-medium text-neutral-900 dark:text-neutral-100 hover:underline'
                      >
                        {template.name}
                      </Link>
                      <Pill tone='neutral' mono>
                        {template.email_type}
                      </Pill>
                      {uses.length > 0 && <Pill tone='success'>assigned</Pill>}
                    </div>
                    <p className='mt-0.5 truncate text-xs text-neutral-500 dark:text-neutral-400'>
                      {spec.label} · updated {formatRelative(template.updated_at || template.created_at)}
                    </p>
                  </div>

                  <Button variant='ghost' size='sm' className='text-xs' onClick={() => onEdit(template.id)}>
                    <Pencil /> Edit
                  </Button>
                  <Button
                    variant='ghost'
                    size='sm'
                    className='text-xs'
                    onClick={() => onExport(template.id, 'json')}
                  >
                    <Download /> JSON
                  </Button>
                  <Button
                    variant='ghost'
                    size='sm'
                    className='font-mono-ui text-xs'
                    onClick={() => onExport(template.id, 'mjml')}
                  >
                    MJML
                  </Button>
                  <Button
                    variant='ghost'
                    size='icon'
                    aria-label={`Delete ${template.name}`}
                    className={uses.length > 0 ? 'text-fk-amber' : 'text-neutral-400 dark:text-neutral-500'}
                    onClick={() => setPendingDelete(template)}
                  >
                    <Trash2 className='size-4' />
                  </Button>
                </div>
              )
            })
          ) : (
            <div className='grid place-items-center rounded-sm border border-dashed border-fk-line px-6 py-12'>
              <Mail className='size-6 text-neutral-300 dark:text-neutral-600' strokeWidth={1.5} />
              <p className='mt-3 text-sm font-medium text-neutral-700 dark:text-neutral-300'>
                {templates.length === 0 ? 'No email template' : 'No template matches the filter'}
              </p>
              <p className='mt-1 max-w-sm text-center text-xs text-neutral-500 dark:text-neutral-400'>
                {templates.length === 0
                  ? 'Until a template is assigned, FerrisKey sends its plain-text default emails.'
                  : 'Adjust the search or the type filter.'}
              </p>
              <div className='mt-3 flex gap-2'>
                {templates.length > 0 && (
                  <Button
                    variant='outline'
                    size='sm'
                    onClick={() => {
                      setQuery('')
                      setTypeFilter('all')
                    }}
                  >
                    Clear filters
                  </Button>
                )}
                <Button size='sm' onClick={onCreate}>
                  <Plus /> New template
                </Button>
              </div>
            </div>
          )}
        </Section>
      </div>

      <ConfirmDeleteAlert
        open={Boolean(pendingDelete)}
        title='Delete email template'
        description={
          pendingUses.length > 0
            ? `"${pendingDelete?.name}" is assigned to ${pendingUses
                .map((spec) => spec.label)
                .join(', ')}. Deleting it succeeds and unassigns it: those emails fall back to the plain-text default.`
            : `This permanently deletes "${pendingDelete?.name}", its structure and its MJML.`
        }
        onConfirm={() => {
          if (pendingDelete) onDelete(pendingDelete.id)
          setPendingDelete(undefined)
        }}
        onCancel={() => setPendingDelete(undefined)}
      />
    </>
  )
}
