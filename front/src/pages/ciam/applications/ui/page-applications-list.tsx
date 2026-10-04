import { createElement } from 'react'
import { useTranslation } from 'react-i18next'
import { Plus } from 'lucide-react'
import { Button } from '@/components/kit/button'
import { CreatePickerDialog, IconTile, ListingPage, Pill, StatusDot } from '@/components/kit'
import type {
  CardSpec,
  ChartTone,
  Column,
  FilterField,
  ListingMetric,
  PagedListing,
  PaginationMetadata,
  PillTone,
  ViewMode,
} from '@/components/kit'
import { formatDate } from '@/utils/format-date'
import { Schemas } from '@/api/api.client'
import {
  applicationTypeChoices,
  applicationTypeIcon,
  applicationTypeMeta,
  applicationTypeMetas,
  inferApplicationType,
  type ApplicationType,
} from '../application-types'

import Client = Schemas.Client

export type ApplicationCounts = Record<ApplicationType | 'total', number>

export interface ApplicationPreview {
  total: number
  names: string[]
}

export interface PageApplicationsListProps {
  applications: Client[]
  pagination: PaginationMetadata | undefined
  listing: PagedListing
  isLoading: boolean
  counts: ApplicationCounts
  missingCallback: ApplicationPreview
  pickerOpen: boolean
  onPickerOpenChange: (open: boolean) => void
  createUrl: (type: ApplicationType) => string
  applicationHref: (application: Client) => string
}

type TileTone = 'info' | 'success' | 'violet' | 'amber' | 'danger' | 'primary'

const tileTone = (tone: PillTone): TileTone => (tone === 'neutral' ? 'info' : tone)

const metricTone = (tone: PillTone): ChartTone =>
  tone === 'neutral' || tone === 'danger' ? 'info' : tone

const DEFAULT_PICKER_TYPE: ApplicationType = 'spa'
const DEFAULT_VIEW: ViewMode = 'cards'

const NAME_SEPARATOR = ', '
const TRUNCATION_MARK = '…'

const displayName = (application: Client) => application.name || application.client_id

export default function PageApplicationsList({
  applications,
  pagination,
  listing,
  isLoading,
  counts,
  missingCallback,
  pickerOpen,
  onPickerOpenChange,
  createUrl,
  applicationHref,
}: PageApplicationsListProps) {
  const { t } = useTranslation('console')

  const metaOf = (application: Client) => applicationTypeMeta(inferApplicationType(application), t)

  const typeIcon = (application: Client) => {
    const meta = metaOf(application)
    return (
      <IconTile tone={tileTone(meta.tone)}>
        {createElement(applicationTypeIcon(meta.key), { className: 'size-4', strokeWidth: 1.75 })}
      </IconTile>
    )
  }

  const columns: Column<Client>[] = [
    {
      key: 'name',
      header: t('applications.list.columns.name'),
      render: (c) => displayName(c),
      sortKey: 'name',
    },
    {
      key: 'client_id',
      header: t('applications.list.columns.client_id'),
      render: (c) => (
        <span className='font-mono-ui text-xs text-neutral-500 dark:text-neutral-400'>
          {c.client_id}
        </span>
      ),
      sortKey: 'client_id',
    },
    {
      key: 'type',
      header: t('applications.list.columns.type'),
      render: (c) => {
        const meta = metaOf(c)
        return (
          <Pill tone={meta.tone} mono>
            {meta.short.toLowerCase()}
          </Pill>
        )
      },
    },
    {
      key: 'status',
      header: t('applications.list.columns.status'),
      render: (c) => (
        <span className='inline-flex items-center gap-1.5 text-xs text-neutral-600 dark:text-neutral-400'>
          <StatusDot on={c.enabled} />
          {c.enabled
            ? t('applications.list.status.enabled')
            : t('applications.list.status.disabled')}
        </span>
      ),
      sortKey: 'enabled',
    },
    {
      key: 'created',
      header: t('applications.list.columns.created'),
      render: (c) => (
        <span className='tnum text-neutral-600 dark:text-neutral-400'>
          {formatDate(c.created_at)}
        </span>
      ),
      sortKey: 'created_at',
    },
  ]

  const filterFields: FilterField[] = [
    {
      kind: 'text',
      key: 'search',
      label: t('applications.list.filter_fields.search'),
      placeholder: t('applications.list.search_placeholder'),
    },
    { kind: 'boolean', key: 'enabled', label: t('applications.list.filter_fields.enabled') },
    {
      kind: 'enum',
      key: 'application_type',
      label: t('applications.list.filter_fields.application_type'),
      options: applicationTypeMetas(t).map((meta) => ({ value: meta.key, label: meta.short })),
    },
  ]

  const card: CardSpec<Client> = {
    avatar: typeIcon,
    title: displayName,
    subtitle: (c) => c.client_id,
    badges: (c) => {
      const meta = metaOf(c)
      return (
        <>
          <Pill tone={meta.tone} mono>
            {meta.short.toLowerCase()}
          </Pill>
          {!c.enabled && <Pill tone='neutral'>{t('applications.list.card.disabled')}</Pill>}
        </>
      )
    },
    footer: (c) => (
      <>
        <span className='truncate'>{metaOf(c).flow}</span>
        <span className='ml-auto shrink-0 tnum'>{formatDate(c.created_at)}</span>
      </>
    ),
  }

  const metrics: ListingMetric[] = [
    {
      key: 'total',
      label: t('applications.list.all'),
      value: counts.total,
      series: [counts.total, counts.total],
      tone: 'primary',
    },
    ...applicationTypeMetas(t).map((meta) => ({
      key: meta.key,
      label: meta.short,
      value: counts[meta.key],
      series: [counts[meta.key], counts[meta.key]],
      tone: metricTone(meta.tone),
    })),
  ]

  const missingNames =
    missingCallback.names.join(NAME_SEPARATOR) +
    (missingCallback.total > missingCallback.names.length ? TRUNCATION_MARK : '')

  const createButton = (
    <Button onClick={() => onPickerOpenChange(true)}>
      <Plus /> {t('applications.list.create')}
    </Button>
  )

  return (
    <>
      <ListingPage
        title={t('applications.list.title')}
        description={t('applications.list.description')}
        loading={isLoading}
        actions={createButton}
        metrics={metrics}
        alerts={
          missingCallback.total
            ? [
                {
                  tone: 'warn' as const,
                  title: t('applications.list.alerts.missing_callback.title', {
                    count: missingCallback.total,
                  }),
                  detail: t('applications.list.alerts.missing_callback.detail', {
                    names: missingNames,
                  }),
                },
              ]
            : []
        }
        paged={{ listing, pagination, filterFields }}
        rows={applications}
        columns={columns}
        card={card}
        getKey={(c) => c.id}
        getHref={applicationHref}
        emptyLabel={t('applications.list.empty.label')}
        emptyHint={t('applications.list.empty.hint')}
        emptyAction={createButton}
        defaultView={DEFAULT_VIEW}
      />

      <CreatePickerDialog
        open={pickerOpen}
        onOpenChange={onPickerOpenChange}
        title={t('applications.list.picker.title')}
        description={t('applications.list.picker.description')}
        options={applicationTypeChoices(t)}
        defaultValue={DEFAULT_PICKER_TYPE}
        createUrl={createUrl}
      />
    </>
  )
}
