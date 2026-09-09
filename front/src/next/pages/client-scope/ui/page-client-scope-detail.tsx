import { ArrowLeft, KeyRound } from 'lucide-react'
import { Button } from '@/components/kit/button'
import FloatingActionBar from '@/components/ui/floating-action-bar'
import { IconTile, PageTabs, Pill, type PillTone, type TabItem } from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import { Schemas } from '@/api/api.client'
import ClientScopeSettingsTab from './client-scope-settings-tab'
import ClientScopeMappersTab from './client-scope-mappers-tab'
import type { ScopeTypeChoice } from './page-create-client-scope'

import ClientScope = Schemas.ClientScope
import ProtocolMapper = Schemas.ProtocolMapper
import ScopeType = Schemas.ScopeType

const typeLabel: Record<ScopeType, string> = {
  DEFAULT: 'default',
  OPTIONAL: 'optional',
  NONE: 'none',
}

const typeTone: Record<ScopeType, PillTone> = {
  DEFAULT: 'success',
  OPTIONAL: 'info',
  NONE: 'neutral',
}

const formatDateTime = (iso: string) =>
  new Date(iso).toLocaleString('en-GB', {
    day: '2-digit',
    month: '2-digit',
    year: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  })

export interface PageClientScopeDetailProps {
  scope?: ClientScope
  isLoading: boolean
  tab: string
  tabs: TabItem[]
  name: string
  description: string
  scopeType: ScopeTypeChoice
  nameError?: string
  dirtyCount: number
  isPending: boolean
  mapperHref: (mapper: ProtocolMapper) => string
  pickerOpen: boolean
  onPickerOpenChange: (open: boolean) => void
  onSelectTemplate: (templateId: string) => void
  onDeleteMapper: (mapper: ProtocolMapper) => void
  onNameChange: (v: string) => void
  onDescriptionChange: (v: string) => void
  onScopeTypeChange: (v: ScopeTypeChoice) => void
  onBack: () => void
  onDiscard: () => void
  onSave: () => void
  onDelete: () => void
}

export default function PageClientScopeDetail({
  scope,
  isLoading,
  tab,
  tabs,
  name,
  description,
  scopeType,
  nameError,
  dirtyCount,
  isPending,
  mapperHref,
  pickerOpen,
  onPickerOpenChange,
  onSelectTemplate,
  onDeleteMapper,
  onNameChange,
  onDescriptionChange,
  onScopeTypeChange,
  onBack,
  onDiscard,
  onSave,
  onDelete,
}: PageClientScopeDetailProps) {
  const container = cn('mx-auto', tokens.page.maxWidth, tokens.page.padding)

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

  if (!scope) {
    return (
      <div className={container}>
        <Button variant='ghost' size='sm' className='-ml-2 mb-2 text-neutral-500' onClick={onBack}>
          <ArrowLeft className='size-3.5' />
          Client Scopes
        </Button>
        <div className={cn(tokens.surface.panel, 'grid place-items-center px-6 py-16')}>
          <p className='text-sm font-medium text-neutral-700'>Client scope not found</p>
          <p className='mt-1 max-w-sm text-center text-sm text-neutral-500'>
            It may have been deleted, or it belongs to another realm.
          </p>
        </div>
      </div>
    )
  }

  const mappers = scope.protocol_mappers ?? []

  return (
    <div className={container}>
      <Button variant='ghost' size='sm' className='-ml-2 mb-2 text-neutral-500' onClick={onBack}>
        <ArrowLeft className='size-3.5' />
        Client Scopes
      </Button>

      <div className='flex flex-wrap items-start justify-between gap-4'>
        <div className='flex items-center gap-3'>
          <IconTile tone='info' className='size-15'>
            <KeyRound className='size-6' strokeWidth={1.75} />
          </IconTile>
          <div className='min-w-0'>
            <h1 className={tokens.header.title}>{scope.name}</h1>
            <div className='mt-1.5 flex flex-wrap items-center gap-2'>
              <Pill tone={typeTone[scope.default_scope_type]} mono>
                {typeLabel[scope.default_scope_type]}
              </Pill>
              <Pill mono>{scope.protocol}</Pill>
              <span className='tnum text-xs text-neutral-500'>
                {mappers.length} mapper{mappers.length !== 1 ? 's' : ''}
              </span>
            </div>
            <p className='mt-1 text-xs text-neutral-500'>
              {scope.description || (
                <span className='font-mono-ui text-neutral-400'>scope_id: {scope.id}</span>
              )}
            </p>
          </div>
        </div>

        <dl className='shrink-0 text-right text-xs text-neutral-500'>
          <dt className='sr-only'>Created at</dt>
          <dd className='tnum'>Created {formatDateTime(scope.created_at)}</dd>
          <dt className='sr-only'>Updated at</dt>
          <dd className='tnum'>Updated {formatDateTime(scope.updated_at)}</dd>
          <dt className='sr-only'>Identifier</dt>
          <dd className='font-mono-ui text-[11px] text-neutral-400'>{scope.id}</dd>
        </dl>
      </div>

      <PageTabs tabs={tabs} value={tab} className='mt-5'>
        <div className={tokens.page.blockGap}>
          {tab === 'details' && (
            <ClientScopeSettingsTab
              scope={scope}
              name={name}
              description={description}
              scopeType={scopeType}
              nameError={nameError}
              onNameChange={onNameChange}
              onDescriptionChange={onDescriptionChange}
              onScopeTypeChange={onScopeTypeChange}
              onDelete={onDelete}
            />
          )}

          {tab === 'mappers' && (
            <ClientScopeMappersTab
              mappers={mappers}
              mapperHref={mapperHref}
              pickerOpen={pickerOpen}
              onPickerOpenChange={onPickerOpenChange}
              onSelectTemplate={onSelectTemplate}
              onDeleteMapper={onDeleteMapper}
            />
          )}
        </div>
      </PageTabs>

      <FloatingActionBar
        show={dirtyCount > 0 && !nameError}
        title={`${dirtyCount} unsaved change${dirtyCount > 1 ? 's' : ''}`}
        description='Review the client scope before applying the changes.'
        onCancel={onDiscard}
        cancelLabel='Discard'
        actions={[{ label: isPending ? 'Saving…' : 'Save changes', onClick: onSave }]}
      />
    </div>
  )
}
