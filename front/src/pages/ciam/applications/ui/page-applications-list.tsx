import { createElement, useMemo, useState } from 'react'
import { Link } from 'react-router-dom'
import { Boxes, Plus, Search, TriangleAlert } from 'lucide-react'
import { Button } from '@/components/kit/button'
import { CreatePickerDialog, EmptyState, IconTile, PageShell, Pill } from '@/components/kit'
import type { PillTone } from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import { formatDate } from '@/utils/format-date'
import { Schemas } from '@/api/api.client'
import {
  APPLICATION_TYPE_METAS,
  applicationTypeChoices,
  applicationTypeIcon,
  applicationTypeMeta,
  inferApplicationType,
  type ApplicationType,
} from '../application-types'

import Client = Schemas.Client

export interface PageApplicationsListProps {
  applications: Client[]
  isLoading: boolean
  pickerOpen: boolean
  onPickerOpenChange: (open: boolean) => void
  createUrl: (type: ApplicationType) => string
  applicationHref: (application: Client) => string
}

type StatusFilter = 'all' | 'enabled' | 'disabled'
type SortKey = 'recent' | 'oldest' | 'name'

const STATUS_FILTERS: { key: StatusFilter; label: string }[] = [
  { key: 'all', label: 'All' },
  { key: 'enabled', label: 'Enabled' },
  { key: 'disabled', label: 'Disabled' },
]

const SORTS: { key: SortKey; label: string }[] = [
  { key: 'recent', label: 'Most recent' },
  { key: 'oldest', label: 'Oldest first' },
  { key: 'name', label: 'Name' },
]

type TileTone = 'info' | 'success' | 'violet' | 'amber' | 'danger' | 'primary'

const tileTone = (tone: PillTone): TileTone => (tone === 'neutral' ? 'info' : tone)

const callbackCount = (application: Client) => application.redirect_uris?.length ?? 0

const displayName = (application: Client) => application.name || application.client_id

const needsCallback = (application: Client) =>
  applicationTypeMeta(inferApplicationType(application)).usesAuthorizationCode &&
  callbackCount(application) === 0

export default function PageApplicationsList({
  applications,
  isLoading,
  pickerOpen,
  onPickerOpenChange,
  createUrl,
  applicationHref,
}: PageApplicationsListProps) {
  const [type, setType] = useState<ApplicationType | 'all'>('all')
  const [status, setStatus] = useState<StatusFilter>('all')
  const [query, setQuery] = useState('')
  const [sort, setSort] = useState<SortKey>('recent')

  const countFor = (key: ApplicationType) =>
    applications.filter((c) => inferApplicationType(c) === key).length

  const rows = useMemo(() => {
    const q = query.trim().toLowerCase()
    const filtered = applications.filter((c) => {
      if (type !== 'all' && inferApplicationType(c) !== type) return false
      if (status === 'enabled' && !c.enabled) return false
      if (status === 'disabled' && c.enabled) return false
      if (q && !`${c.name} ${c.client_id}`.toLowerCase().includes(q)) return false
      return true
    })

    return [...filtered].sort((a, b) => {
      if (sort === 'name') return displayName(a).localeCompare(displayName(b))
      const delta = new Date(a.created_at).getTime() - new Date(b.created_at).getTime()
      return sort === 'oldest' ? delta : -delta
    })
  }, [applications, type, status, query, sort])

  const missingCallback = applications.filter(needsCallback)

  const createButton = (
    <Button onClick={() => onPickerOpenChange(true)}>
      <Plus /> Create application
    </Button>
  )

  return (
    <PageShell className={tokens.page.sectionGap}>
      <div
        className={cn('flex flex-wrap items-start justify-between gap-3', tokens.header.spacing)}
      >
        <div className='min-w-0'>
          <h1 className={tokens.header.title}>Applications</h1>
          <p className='mt-0.5 text-sm text-neutral-500 dark:text-neutral-400'>
            Anything that talks to FerrisKey: mobile apps, single-page apps, web servers, daemons.
          </p>
        </div>
        <div className='flex shrink-0 items-center gap-2'>{createButton}</div>
      </div>

      <div className='grid grid-cols-2 gap-2 sm:grid-cols-3 lg:grid-cols-6'>
        <TypeCard
          label='All applications'
          value={applications.length}
          icon={Boxes}
          tone='primary'
          active={type === 'all'}
          onSelect={() => setType('all')}
        />
        {APPLICATION_TYPE_METAS.map((meta) => (
          <TypeCard
            key={meta.key}
            label={meta.short}
            value={countFor(meta.key)}
            icon={applicationTypeIcon(meta.key)}
            tone={tileTone(meta.tone)}
            active={type === meta.key}
            onSelect={() => setType(meta.key)}
          />
        ))}
      </div>

      {missingCallback.length > 0 && (
        <div className='flex items-start gap-2 rounded-md border border-fk-amber-border bg-fk-amber-soft/40 px-3 py-2 text-xs'>
          <TriangleAlert className='mt-0.5 size-3.5 shrink-0 text-fk-amber' />
          <p className='text-neutral-600 dark:text-neutral-400'>
            <span className='font-medium text-neutral-900 dark:text-neutral-100'>
              {missingCallback.length} application{missingCallback.length > 1 ? 's have' : ' has'} no
              callback URL
            </span>{' '}
            — {missingCallback.map(displayName).join(', ')}: FerrisKey has nowhere to send the user
            back, so sign-in stops with an error.
          </p>
        </div>
      )}

      <div className='flex flex-wrap items-center justify-between gap-2'>
        <div className='flex items-center gap-1'>
          {STATUS_FILTERS.map((filter) => (
            <button
              key={filter.key}
              type='button'
              onClick={() => setStatus(filter.key)}
              className={cn(
                'cursor-pointer rounded-md border px-2.5 py-1 text-xs font-medium transition-colors',
                status === filter.key
                  ? 'border-fk-primary-border bg-fk-primary-soft text-fk-primary-text'
                  : 'border-fk-line text-neutral-600 hover:bg-neutral-50 dark:text-neutral-400 dark:hover:bg-fk-raised'
              )}
            >
              {filter.label}
            </button>
          ))}
        </div>

        <div className='flex items-center gap-2'>
          <label className='relative flex h-8 w-64 items-center'>
            <Search className='pointer-events-none absolute left-2 size-3.5 text-neutral-400 dark:text-neutral-500' />
            <input
              type='search'
              value={query}
              onChange={(event) => setQuery(event.target.value)}
              placeholder='Search by name or client ID…'
              className='h-full w-full rounded-md border border-fk-line bg-transparent pl-7 pr-2 text-xs outline-none placeholder:text-neutral-400 focus:border-fk-primary-border dark:placeholder:text-neutral-500'
            />
          </label>
          <select
            value={sort}
            onChange={(event) => setSort(event.target.value as SortKey)}
            className='h-8 cursor-pointer rounded-md border border-fk-line bg-transparent px-2 text-xs text-neutral-600 outline-none focus:border-fk-primary-border dark:text-neutral-400'
          >
            {SORTS.map((option) => (
              <option key={option.key} value={option.key}>
                {option.label}
              </option>
            ))}
          </select>
        </div>
      </div>

      {isLoading ? (
        <div className='grid gap-3 sm:grid-cols-2 xl:grid-cols-3'>
          {Array.from({ length: 6 }).map((_, index) => (
            <div key={index} className={cn(tokens.surface.panel, 'h-[104px] animate-pulse')} />
          ))}
        </div>
      ) : rows.length === 0 ? (
        <EmptyState
          label={applications.length === 0 ? 'No application yet' : 'No application matches'}
          hint={
            applications.length === 0
              ? 'Pick a type, give it a name, and we generate everything you need to integrate.'
              : 'No application of this type carries that name or client ID.'
          }
          action={applications.length === 0 ? createButton : undefined}
        />
      ) : (
        <div className='grid gap-3 sm:grid-cols-2 xl:grid-cols-3'>
          {rows.map((application) => (
            <ApplicationCard
              key={application.id}
              application={application}
              href={applicationHref(application)}
            />
          ))}
        </div>
      )}

      <CreatePickerDialog
        open={pickerOpen}
        onOpenChange={onPickerOpenChange}
        title='Application type'
        description='It decides which sign-in flow the application uses and whether it gets a secret. Changing it later means recreating the application.'
        options={applicationTypeChoices}
        defaultValue='spa'
        createUrl={createUrl}
      />
    </PageShell>
  )
}

function TypeCard({
  label,
  value,
  icon: Icon,
  tone,
  active,
  onSelect,
}: {
  label: string
  value: number
  icon: React.ComponentType<{ className?: string; strokeWidth?: number }>
  tone: TileTone
  active: boolean
  onSelect: () => void
}) {
  return (
    <button
      type='button'
      onClick={onSelect}
      aria-pressed={active}
      className={cn(
        'flex cursor-pointer items-center gap-2.5 rounded-sm border px-3 py-2.5 text-left transition-colors',
        active
          ? 'border-fk-primary bg-fk-primary-soft/40'
          : 'border-fk-line bg-white hover:bg-neutral-50 dark:bg-fk-surface dark:hover:bg-fk-raised'
      )}
    >
      <IconTile tone={tone}>
        <Icon className='size-4' strokeWidth={1.75} />
      </IconTile>
      <span className='min-w-0'>
        <span className='block tnum text-lg font-semibold leading-none text-neutral-900 dark:text-neutral-100'>
          {value}
        </span>
        <span className='mt-1 block truncate text-xs text-neutral-500 dark:text-neutral-400'>
          {label}
        </span>
      </span>
    </button>
  )
}

function ApplicationCard({ application, href }: { application: Client; href: string }) {
  const meta = applicationTypeMeta(inferApplicationType(application))

  return (
    <Link
      to={href}
      className={cn(
        tokens.surface.panel,
        'block p-0 transition-colors hover:border-fk-primary-border focus-visible:outline-none focus-visible:ring-3 focus-visible:ring-fk-primary/30'
      )}
    >
      <div className='flex items-center gap-2.5 px-3 py-2.5'>
        <IconTile tone={tileTone(meta.tone)}>
          {createElement(applicationTypeIcon(meta.key), {
            className: 'size-4',
            strokeWidth: 1.75,
          })}
        </IconTile>
        <span className='min-w-0'>
          <span className='block truncate text-sm font-medium text-neutral-900 dark:text-neutral-100'>
            {displayName(application)}
          </span>
          <span className='block truncate font-mono-ui text-[11px] text-neutral-400 dark:text-neutral-500'>
            {application.client_id}
          </span>
        </span>
      </div>
      <div className='flex items-center gap-2 border-t border-fk-line px-3 py-2 text-xs'>
        <Pill tone={meta.tone} mono>
          {meta.short.toLowerCase()}
        </Pill>
        <span className='truncate text-neutral-500 dark:text-neutral-400'>{meta.flow}</span>
        {!application.enabled && <Pill tone='neutral'>disabled</Pill>}
        <span className='ml-auto shrink-0 tnum text-neutral-400 dark:text-neutral-500'>
          {formatDate(application.created_at)}
        </span>
      </div>
    </Link>
  )
}
