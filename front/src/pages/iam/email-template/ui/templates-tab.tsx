import { useMemo, useState } from 'react'
import { Link } from 'react-router-dom'
import { AlertTriangle, Download, Mail, Pencil, Plus, Search, Trash2 } from 'lucide-react'
import { useTranslation } from 'react-i18next'
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
import {
  ALL_EMAIL_TYPES,
  EMAIL_TEMPLATE_NAMESPACE,
  EMAIL_TYPES,
  EXPORT_JSON,
  EXPORT_MJML,
  emailTypeSpec,
  formatRelative,
  type EmailTypeSpec,
} from '../email-types'

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

const LIST_SEPARATOR = ', '

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
  const { t } = useTranslation(EMAIL_TEMPLATE_NAMESPACE)
  const [query, setQuery] = useState('')
  const [typeFilter, setTypeFilter] = useState<string>(ALL_EMAIL_TYPES)
  const [pendingDelete, setPendingDelete] = useState<EmailTemplate | undefined>(undefined)

  const usedBy = (templateId: string) =>
    EMAIL_TYPES.filter((spec) => assignments[spec.assignmentField] === templateId)

  const nameList = (specs: EmailTypeSpec[]) =>
    specs.map((spec) => t(spec.labelKey)).join(LIST_SEPARATOR)

  const rows = useMemo(() => {
    const needle = query.trim().toLowerCase()
    const matching = templates.filter((template) => {
      if (typeFilter !== ALL_EMAIL_TYPES && template.email_type !== typeFilter) return false
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
              {t('list.alert.unassigned', {
                count: unassigned.length,
                names: nameList(unassigned),
              })}
            </p>
          </div>
        )}

        <MetricsBand
          metrics={[
            {
              key: 'total',
              label: t('list.metrics.total.label'),
              value: templates.length,
              hint: t('list.metrics.total.hint'),
            },
            ...EMAIL_TYPES.map((spec) => ({
              key: spec.key,
              label: t(spec.shortKey),
              value: templates.filter((template) => template.email_type === spec.key).length,
              hint: assignments[spec.assignmentField]
                ? t('list.metrics.type.assigned')
                : t('list.metrics.type.none'),
            })),
          ]}
        />

        <Section
          title={t('list.assignment.title')}
          description={t('list.assignment.description')}
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
                    <p className='text-sm font-medium text-neutral-900 dark:text-neutral-100'>
                      {t(spec.labelKey)}
                    </p>
                    <p className='mt-0.5 text-xs leading-relaxed text-neutral-500 dark:text-neutral-400'>
                      {t(spec.triggerKey)}
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
                      <SelectItem value={NONE}>{t('list.assignment.not_configured')}</SelectItem>
                      {candidates.map((template) => (
                        <SelectItem key={template.id} value={template.id}>
                          {template.name}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                  {candidates.length === 0 && (
                    <p className='mt-1.5 text-xs text-neutral-400 dark:text-neutral-500'>
                      {t('list.assignment.empty')}
                    </p>
                  )}
                </div>
              </div>
            )
          })}
        </Section>

        <Section
          title={t('list.templates.title')}
          description={t('list.templates.description')}
          action={
            <div className='flex items-center gap-2'>
              <label className='relative flex h-8 w-52 items-center'>
                <Search className='pointer-events-none absolute left-2.5 size-3.5 text-neutral-400 dark:text-neutral-500' />
                <input
                  type='search'
                  value={query}
                  onChange={(event) => setQuery(event.target.value)}
                  placeholder={t('list.templates.search_placeholder')}
                  className='h-full w-full rounded-md border border-fk-line pl-8 pr-2 text-[13px] outline-none placeholder:text-neutral-400 focus:border-fk-primary-border'
                />
              </label>
              <div className='flex overflow-hidden rounded-md border border-fk-line'>
                {[ALL_EMAIL_TYPES, ...EMAIL_TYPES.map((spec) => spec.key)].map((key) => (
                  <button
                    key={key}
                    type='button'
                    onClick={() => setTypeFilter(key)}
                    aria-pressed={typeFilter === key}
                    className={cn(
                      'px-2 py-1 text-xs transition-colors',
                      typeFilter === key
                        ? 'bg-fk-primary-soft font-medium text-fk-primary-text'
                        : 'text-neutral-500 hover:bg-neutral-50 dark:text-neutral-400 dark:hover:bg-fk-surface'
                    )}
                  >
                    {key === ALL_EMAIL_TYPES
                      ? t('list.templates.filter_all')
                      : t(emailTypeSpec(key).shortKey)}
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
                <div className='size-9 animate-pulse rounded-md bg-neutral-100 dark:bg-fk-raised' />
                <div className='flex-1 space-y-2'>
                  <div className='h-3.5 w-48 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
                  <div className='h-3 w-32 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
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
                      {uses.length > 0 && <Pill tone='success'>{t('list.templates.assigned')}</Pill>}
                    </div>
                    <p className='mt-0.5 truncate text-xs text-neutral-500 dark:text-neutral-400'>
                      {t('list.templates.meta', {
                        type: t(spec.labelKey),
                        when: formatRelative(template.updated_at || template.created_at),
                      })}
                    </p>
                  </div>

                  <Button variant='ghost' size='sm' className='text-xs' onClick={() => onEdit(template.id)}>
                    <Pencil /> {t('list.templates.edit')}
                  </Button>
                  <Button
                    variant='ghost'
                    size='sm'
                    className='text-xs'
                    onClick={() => onExport(template.id, EXPORT_JSON)}
                  >
                    <Download /> {t('list.templates.export_json')}
                  </Button>
                  <Button
                    variant='ghost'
                    size='sm'
                    className='font-mono-ui text-xs'
                    onClick={() => onExport(template.id, EXPORT_MJML)}
                  >
                    {t('list.templates.export_mjml')}
                  </Button>
                  <Button
                    variant='ghost'
                    size='icon'
                    aria-label={t('list.templates.delete_aria', { name: template.name })}
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
                {templates.length === 0
                  ? t('list.empty.none.label')
                  : t('list.empty.filtered.label')}
              </p>
              <p className='mt-1 max-w-sm text-center text-xs text-neutral-500 dark:text-neutral-400'>
                {templates.length === 0
                  ? t('list.empty.none.hint')
                  : t('list.empty.filtered.hint')}
              </p>
              <div className='mt-3 flex gap-2'>
                {templates.length > 0 && (
                  <Button
                    variant='outline'
                    size='sm'
                    onClick={() => {
                      setQuery('')
                      setTypeFilter(ALL_EMAIL_TYPES)
                    }}
                  >
                    {t('list.empty.clear')}
                  </Button>
                )}
                <Button size='sm' onClick={onCreate}>
                  <Plus /> {t('list.empty.create')}
                </Button>
              </div>
            </div>
          )}
        </Section>
      </div>

      <ConfirmDeleteAlert
        open={Boolean(pendingDelete)}
        title={t('list.delete.title')}
        description={
          pendingUses.length > 0
            ? t('list.delete.assigned', {
                count: pendingUses.length,
                name: pendingDelete?.name,
                targets: nameList(pendingUses),
              })
            : t('list.delete.plain', { name: pendingDelete?.name })
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
