import { Pill, Section } from '@/components/kit'
import { cn } from '@/lib/utils'
import { Schemas } from '@/api/api.client'
import {
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
        title='Last synchronisation'
        description={lastSync ? `Started on ${lastSync}.` : 'Never started.'}
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
                  finished {formatSyncedAt(lastRun.completed_at)}
                </span>
              )}
            </div>

            <div className='grid grid-cols-2 gap-2 pb-3 sm:grid-cols-5'>
              {(
                [
                  ['Processed', lastRun.total_processed, false],
                  ['Created', lastRun.created, false],
                  ['Updated', lastRun.updated, false],
                  ['Disabled', lastRun.disabled, false],
                  ['Failed', lastRun.failed, true],
                ] as const
              ).map(([label, value, danger]) => (
                <div key={label} className='rounded-md border border-fk-line px-2.5 py-2'>
                  <p className='text-[11px] text-neutral-500 dark:text-neutral-400'>{label}</p>
                  <p
                    className={cn(
                      'tnum mt-0.5 text-lg font-semibold leading-none',
                      danger && value > 0 ? 'text-fk-danger' : 'text-neutral-900 dark:text-neutral-100'
                    )}
                  >
                    {value}
                  </p>
                </div>
              ))}
            </div>
          </>
        ) : provider.last_sync_at ? (
          <div className='flex flex-wrap items-center gap-2 py-3'>
            {provider.last_sync_status ? (
              <Pill tone={statusTone(provider.last_sync_status)} mono>
                {provider.last_sync_status}
              </Pill>
            ) : (
              <Pill mono>unknown outcome</Pill>
            )}
            <span className='text-xs text-neutral-500 dark:text-neutral-400'>
              Run this provider from the header to see what a pass creates, updates and fails on.
            </span>
          </div>
        ) : (
          <p className='rounded-lg border border-dashed border-fk-amber-border bg-fk-amber-soft/40 px-4 py-3 text-xs text-neutral-600 dark:text-neutral-400'>
            This provider has never synchronised — no account has been imported so far.
          </p>
        )}
      </Section>
    </>
  )
}
