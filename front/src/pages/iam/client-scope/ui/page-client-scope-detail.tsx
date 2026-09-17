import { ArrowLeft, KeyRound } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/kit/button'
import SaveBar from '@/components/kit/save-bar'
import { DetailHeader, IconTile, PageShell, PageTabs, Pill, type TabItem } from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import { Schemas } from '@/api/api.client'
import ClientScopeSettingsTab from './client-scope-settings-tab'
import ClientScopeMappersTab from './client-scope-mappers-tab'
import type { ScopeTypeChoice } from './page-create-client-scope'

import ClientScope = Schemas.ClientScope
import ProtocolMapper = Schemas.ProtocolMapper
import { formatDateTime } from '@/utils/format-date'
import { SCOPE_TYPE_TONE, scopeTypeLabelKey } from '../scope-type'

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
  const { t } = useTranslation('client-scope')

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

  if (!scope) {
    return (
      <PageShell>
        <Button variant='ghost' size='sm' className='-ml-2 mb-2 text-neutral-500 dark:text-neutral-400' onClick={onBack}>
          <ArrowLeft className='size-3.5' />
          {t('detail.back')}
        </Button>
        <div className={cn(tokens.surface.panel, 'grid place-items-center px-6 py-16')}>
          <p className='text-sm font-medium text-neutral-700 dark:text-neutral-300'>
            {t('detail.not_found.title')}
          </p>
          <p className='mt-1 max-w-sm text-center text-sm text-neutral-500 dark:text-neutral-400'>
            {t('detail.not_found.description')}
          </p>
        </div>
      </PageShell>
    )
  }

  const mappers = scope.protocol_mappers ?? []

  return (
    <PageShell>
      <DetailHeader
        onBack={onBack}
        backLabel={t('detail.back')}
        icon={
          <IconTile tone='info' className='size-15'>
            <KeyRound className='size-6' strokeWidth={1.75} />
          </IconTile>
        }
        title={scope.name}
        pills={
          <>
            <Pill tone={SCOPE_TYPE_TONE[scope.default_scope_type]} mono>
              {t(scopeTypeLabelKey(scope.default_scope_type))}
            </Pill>
            <Pill mono>{scope.protocol}</Pill>
            <span className='tnum text-xs text-neutral-500 dark:text-neutral-400'>
              {t('scope.mapper_count', { count: mappers.length })}
            </span>
            <p className='mt-1 w-full text-xs text-neutral-500 dark:text-neutral-400'>
              {scope.description || (
                <span className='font-mono-ui text-neutral-400 dark:text-neutral-500'>
                  {t('scope.identifier', { id: scope.id })}
                </span>
              )}
            </p>
          </>
        }
        meta={
          <dl className='shrink-0 text-right text-xs text-neutral-500 dark:text-neutral-400'>
            <dt className='sr-only'>{t('detail.meta.created_label')}</dt>
            <dd className='tnum'>
              {t('detail.meta.created', { date: formatDateTime(scope.created_at) })}
            </dd>
            <dt className='sr-only'>{t('detail.meta.updated_label')}</dt>
            <dd className='tnum'>
              {t('detail.meta.updated', { date: formatDateTime(scope.updated_at) })}
            </dd>
            <dt className='sr-only'>{t('detail.meta.identifier_label')}</dt>
            <dd className='font-mono-ui text-[11px] text-neutral-400 dark:text-neutral-500'>{scope.id}</dd>
          </dl>
        }
      />

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

      <SaveBar
        show={dirtyCount > 0 && !nameError}
        title={t('detail.save_bar.title', { count: dirtyCount })}
        description={t('detail.save_bar.description')}
        onCancel={onDiscard}
        cancelLabel={t('detail.save_bar.cancel')}
        actions={[
          {
            label: isPending ? t('detail.save_bar.submitting') : t('detail.save_bar.submit'),
            onClick: onSave,
          },
        ]}
      />
    </PageShell>
  )
}
