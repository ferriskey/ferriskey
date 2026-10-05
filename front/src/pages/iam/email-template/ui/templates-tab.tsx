import { useState } from 'react'
import { Link } from 'react-router-dom'
import { AlertTriangle, Download, Pencil, Plus, Trash2 } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/kit/button'
import { ConfirmDeleteAlert } from '@/components/confirm-delete-alert'
import {
  DataView,
  IconTile,
  ListingToolbar,
  MetricsBand,
  PaginationBar,
  Pill,
  RelationSelect,
  Section,
  type Column,
  type PagedListing,
  type PaginationMetadata,
  type ViewMode,
} from '@/components/kit'
import { tokens } from '@/styles/style-tokens'
import { Schemas } from '@/api/api.client'
import { emailTemplateRelationSources } from '@/api/email-template.relation'
import { formatDate } from '@/utils/format-date'
import {
  EMAIL_TEMPLATE_NAMESPACE,
  EMAIL_TYPES,
  EXPORT_JSON,
  EXPORT_MJML,
  emailTypeSpec,
  formatRelative,
  type EmailTypeSpec,
} from '../email-types'

import EmailTemplate = Schemas.EmailTemplate
import EmailType = Schemas.EmailType

type Assignments = Record<EmailTypeSpec['assignmentField'], string | null>

export interface TemplateCounts {
  total: number
  byType: Record<EmailType, number>
  loadingByType: Record<EmailType, boolean>
}

export interface TemplatesTabProps {
  templates: EmailTemplate[]
  listing: PagedListing
  pagination: PaginationMetadata | undefined
  counts: TemplateCounts
  isLoading: boolean
  assignments: Assignments
  templateHref: (id: string) => string
  onAssign: (field: EmailTypeSpec['assignmentField'], templateId: string | null) => void
  onCreate: () => void
  onEdit: (id: string) => void
  onDelete: (id: string) => void
  onExport: (id: string, format: 'json' | 'mjml') => void
}

const LIST_SEPARATOR = ', '

const TEMPLATES_VIEW: ViewMode = 'list'

export default function TemplatesTab({
  templates,
  listing,
  pagination,
  counts,
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
  const [pendingDelete, setPendingDelete] = useState<EmailTemplate | undefined>(undefined)

  const usedBy = (templateId: string) =>
    EMAIL_TYPES.filter((spec) => assignments[spec.assignmentField] === templateId)

  const nameList = (specs: EmailTypeSpec[]) =>
    specs.map((spec) => t(spec.labelKey)).join(LIST_SEPARATOR)

  const narrowed = Object.values(listing.state.filters).some(Boolean)

  const createButton = (
    <Button size='sm' onClick={onCreate}>
      <Plus /> {t('list.empty.create')}
    </Button>
  )

  const templateIcon = (template: EmailTemplate) => {
    const spec = emailTypeSpec(template.email_type)
    return (
      <IconTile tone={spec.tone}>
        <spec.icon className='size-4' strokeWidth={1.75} />
      </IconTile>
    )
  }

  const assignedPill = (template: EmailTemplate) =>
    usedBy(template.id).length > 0 ? (
      <Pill tone='success'>{t('list.templates.assigned')}</Pill>
    ) : null

  const columns: Column<EmailTemplate>[] = [
    {
      key: 'name',
      header: t('list.columns.name'),
      sortKey: 'name',
      filters: [{ kind: 'text', key: 'name', label: t('list.filters.name') }],
      render: (template) => (
        <div className='flex min-w-0 items-center gap-3'>
          {templateIcon(template)}
          <div className='min-w-0'>
            <div className='flex flex-wrap items-center gap-2'>
              <Link
                to={templateHref(template.id)}
                className='text-[13px] font-medium text-neutral-900 dark:text-neutral-100 hover:underline'
              >
                {template.name}
              </Link>
              {assignedPill(template)}
            </div>
            <p className='mt-0.5 truncate text-xs text-neutral-500 dark:text-neutral-400'>
              {t('list.templates.meta', {
                type: t(emailTypeSpec(template.email_type).labelKey),
                when: formatRelative(template.updated_at || template.created_at),
              })}
            </p>
          </div>
        </div>
      ),
    },
    {
      key: 'email_type',
      header: t('list.columns.email_type'),
      sortKey: 'email_type',
      filters: [
        {
          kind: 'enum',
          key: 'email_type',
          label: t('list.filters.email_type'),
          options: EMAIL_TYPES.map((spec) => ({ value: spec.key, label: t(spec.shortKey) })),
        },
      ],
      render: (template) => (
        <Pill tone='neutral' mono>
          {template.email_type}
        </Pill>
      ),
    },
    {
      key: 'updated_at',
      header: t('list.columns.updated_at'),
      sortKey: 'updated_at',
      render: (template) => (
        <span className='tnum text-xs text-neutral-500 dark:text-neutral-400'>
          {formatDate(template.updated_at)}
        </span>
      ),
    },
    {
      key: 'created_at',
      header: t('list.columns.created_at'),
      sortKey: 'created_at',
      filters: [
        {
          kind: 'date-range',
          fromKey: 'created_from',
          toKey: 'created_to',
          label: t('list.filters.created'),
        },
      ],
      render: (template) => (
        <span className='tnum text-xs text-neutral-500 dark:text-neutral-400'>
          {formatDate(template.created_at)}
        </span>
      ),
    },
    {
      key: 'actions',
      header: '',
      align: 'right',
      render: (template) => (
        <div className='flex items-center justify-end gap-1'>
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
            className={
              usedBy(template.id).length > 0
                ? 'text-fk-amber'
                : 'text-neutral-400 dark:text-neutral-500'
            }
            onClick={() => setPendingDelete(template)}
          >
            <Trash2 className='size-4' />
          </Button>
        </div>
      ),
    },
  ]

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
              value: counts.total,
              hint: t('list.metrics.total.hint'),
              series: [counts.total, counts.total],
              filter: {},
            },
            ...EMAIL_TYPES.map((spec) => ({
              key: spec.key,
              label: t(spec.shortKey),
              value: counts.byType[spec.key],
              hint: assignments[spec.assignmentField]
                ? t('list.metrics.type.assigned')
                : t('list.metrics.type.none'),
              series: [counts.byType[spec.key], counts.byType[spec.key]],
              filter: { email_type: spec.key },
            })),
          ]}
          filtering={{ filters: listing.state.filters, onChange: listing.setFilters }}
        />

        <Section
          title={t('list.assignment.title')}
          description={t('list.assignment.description')}
        >
          {EMAIL_TYPES.map((spec) => (
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
                <RelationSelect
                  source={emailTemplateRelationSources[spec.key]}
                  value={assignments[spec.assignmentField] ?? undefined}
                  onChange={(id) => onAssign(spec.assignmentField, id ?? null)}
                  label={t('list.assignment.picker_label')}
                  noneLabel={t('list.assignment.not_configured')}
                />
                {!counts.loadingByType[spec.key] && counts.byType[spec.key] === 0 && (
                  <p className='mt-1.5 text-xs text-neutral-400 dark:text-neutral-500'>
                    {t('list.assignment.empty')}
                  </p>
                )}
              </div>
            </div>
          ))}
        </Section>

        <Section
          title={t('list.templates.title')}
          description={t('list.templates.description')}
        >
          <ListingToolbar
            listing={listing}
            columns={columns}
            search={{ placeholder: t('list.templates.search_placeholder') }}
            className='mb-3'
          />

          <DataView
            listing={listing}
            rows={templates}
            columns={columns}
            card={{
              avatar: templateIcon,
              title: (template) => template.name,
              subtitle: (template) => t(emailTypeSpec(template.email_type).labelKey),
              badges: assignedPill,
              footer: (template) => <span>{formatDate(template.updated_at)}</span>,
            }}
            getKey={(template) => template.id}
            view={TEMPLATES_VIEW}
            loading={isLoading}
            sort={listing.state.sort}
            onSortChange={listing.setSort}
            emptyLabel={narrowed ? t('list.empty.filtered.label') : t('list.empty.none.label')}
            emptyHint={narrowed ? t('list.empty.filtered.hint') : t('list.empty.none.hint')}
            emptyAction={
              narrowed ? (
                <Button variant='outline' size='sm' onClick={listing.clearFilters}>
                  {t('list.empty.clear')}
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
