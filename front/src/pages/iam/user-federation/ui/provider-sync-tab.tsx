import { useTranslation } from 'react-i18next'
import { Pill, Section } from '@/components/kit'
import { cn } from '@/lib/utils'
import { Schemas } from '@/api/api.client'
import {
  USER_FEDERATION_NAMESPACE,
  formatSyncedAt,
  formatDuration,
  type SyncMode,
} from '../provider-config'
import { SynchronisationFields, type ProviderErrors } from './provider-form-fields'

import ProviderResponse = Schemas.ProviderResponse
import SyncUsersResponse = Schemas.SyncUsersResponse

export interface ProviderSyncTabProps {
  provider: ProviderResponse
  syncEnabled: boolean
  syncMode: SyncMode
  syncIntervalSeconds: number
  errors: ProviderErrors
  lastRun?: SyncUsersResponse
  onSyncEnabledChange: (v: boolean) => void
  onSyncModeChange: (v: SyncMode) => void
  onSyncIntervalChange: (seconds: number) => void
}

const statusTone = (status: string) => {
  const normalized = status.toLowerCase()
  if (normalized.includes('success') && !normalized.includes('partial')) return 'success' as const
  if (normalized.includes('partial')) return 'amber' as const
  if (normalized.includes('progress')) return 'info' as const
  return 'danger' as const
}

const SYNC_STATS = [
  {
    key: 'processed',
    labelKey: 'detail.sync.stats.processed',
    read: (run: SyncUsersResponse) => run.total_processed,
    danger: false,
  },
  {
    key: 'created',
    labelKey: 'detail.sync.stats.created',
    read: (run: SyncUsersResponse) => run.created,
    danger: false,
  },
  {
    key: 'updated',
    labelKey: 'detail.sync.stats.updated',
    read: (run: SyncUsersResponse) => run.updated,
    danger: false,
  },
  {
    key: 'disabled',
    labelKey: 'detail.sync.stats.disabled',
    read: (run: SyncUsersResponse) => run.disabled,
    danger: false,
  },
  {
    key: 'failed',
    labelKey: 'detail.sync.stats.failed',
    read: (run: SyncUsersResponse) => run.failed,
    danger: true,
  },
]

const elapsed = (run: SyncUsersResponse) => {
  if (!run.started_at || !run.completed_at) return null
  const ms = new Date(run.completed_at).getTime() - new Date(run.started_at).getTime()
  return Number.isFinite(ms) && ms >= 0 ? ms : null
}

export default function ProviderSyncTab({
  provider,
  syncEnabled,
  syncMode,
  syncIntervalSeconds,
  errors,
  lastRun,
  onSyncEnabledChange,
  onSyncModeChange,
  onSyncIntervalChange,
}: ProviderSyncTabProps) {
  const { t } = useTranslation(USER_FEDERATION_NAMESPACE)
  const lastSync = formatSyncedAt(provider.last_sync_at)

  return (
    <>
      <SynchronisationFields
        syncEnabled={syncEnabled}
        syncMode={syncMode}
        syncIntervalSeconds={syncIntervalSeconds}
        errors={errors}
        onSyncEnabledChange={onSyncEnabledChange}
        onSyncModeChange={onSyncModeChange}
        onSyncIntervalChange={onSyncIntervalChange}
      />

      <Section
        title={t('detail.sync.last.title')}
        description={
          lastSync
            ? t('detail.sync.last.started', { date: lastSync })
            : t('detail.sync.last.never_started')
        }
        contained={Boolean(lastRun ?? provider.last_sync_at)}
      >
        {lastRun ? (
          <>
            <div className='flex flex-wrap items-center gap-2 py-3'>
              {provider.last_sync_status && (
                <Pill tone={statusTone(provider.last_sync_status)} mono>
                  {provider.last_sync_status}
                </Pill>
              )}
              <span className='tnum text-xs text-neutral-500 dark:text-neutral-400'>
                {formatDuration(elapsed(lastRun))}
              </span>
              {lastRun.completed_at && (
                <span className='text-xs text-neutral-500 dark:text-neutral-400'>
                  {t('detail.sync.last.finished', {
                    date: formatSyncedAt(lastRun.completed_at),
                  })}
                </span>
              )}
            </div>

            <div className='grid grid-cols-2 gap-2 pb-3 sm:grid-cols-5'>
              {SYNC_STATS.map((stat) => {
                const value = stat.read(lastRun)

                return (
                  <div key={stat.key} className='rounded-md border border-fk-line px-2.5 py-2'>
                    <p className='text-[11px] text-neutral-500 dark:text-neutral-400'>
                      {t(stat.labelKey)}
                    </p>
                    <p
                      className={cn(
                        'tnum mt-0.5 text-lg font-semibold leading-none',
                        stat.danger && value > 0
                          ? 'text-fk-danger'
                          : 'text-neutral-900 dark:text-neutral-100'
                      )}
                    >
                      {value}
                    </p>
                  </div>
                )
              })}
            </div>
          </>
        ) : provider.last_sync_at ? (
          <div className='flex flex-wrap items-center gap-2 py-3'>
            {provider.last_sync_status ? (
              <Pill tone={statusTone(provider.last_sync_status)} mono>
                {provider.last_sync_status}
              </Pill>
            ) : (
              <Pill mono>{t('detail.sync.last.unknown_outcome')}</Pill>
            )}
            <span className='text-xs text-neutral-500 dark:text-neutral-400'>
              {t('detail.sync.last.hint')}
            </span>
          </div>
        ) : (
          <p className='rounded-lg border border-dashed border-fk-amber-border bg-fk-amber-soft/40 px-4 py-3 text-xs text-neutral-600 dark:text-neutral-400'>
            {t('detail.sync.last.empty')}
          </p>
        )}
      </Section>
    </>
  )
}
