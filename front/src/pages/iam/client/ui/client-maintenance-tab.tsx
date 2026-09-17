import { ShieldCheck, User, Wrench } from 'lucide-react'
import type { ComponentType } from 'react'
import { useTranslation } from 'react-i18next'
import { Input } from '@/components/ui/input'
import SaveBar from '@/components/kit/save-bar'
import {
  ChoiceCards,
  EntityPicker,
  FieldRow,
  IconTile,
  Pill,
  Section,
  type PickableEntity,
} from '@/components/kit'
import { tokens } from '@/styles/style-tokens'
import { Schemas } from '@/api/api.client'
import {
  isMaintenanceState,
  maintenanceStateChoices,
  maintenanceStateOf,
  sessionStrategyChoices,
} from '../client-choices'

import MaintenanceSessionStrategy = Schemas.MaintenanceSessionStrategy

export interface InheritedEntry {
  id: string
  label: string
  sublabel?: string
}

export interface ClientMaintenanceTabProps {
  enabled: boolean
  reason: string
  strategy: MaintenanceSessionStrategy
  whitelistCount: number
  hasSettingsChanges: boolean
  users: PickableEntity[]
  roles: PickableEntity[]
  whitelistedUserIds: string[]
  whitelistedRoleIds: string[]
  inheritedUsers: InheritedEntry[]
  inheritedRoles: InheritedEntry[]
  onToggle: (enabled: boolean) => void
  onReasonChange: (reason: string) => void
  onStrategyChange: (strategy: MaintenanceSessionStrategy) => void
  onWhitelistedUsersChange: (next: string[]) => void
  onWhitelistedRolesChange: (next: string[]) => void
  onSaveSettings: () => void
  onResetSettings: () => void
}

function InheritedList({
  entries,
  icon: Icon,
  tone,
}: {
  entries: InheritedEntry[]
  icon: ComponentType<{ className?: string; strokeWidth?: number }>
  tone: 'info' | 'violet'
}) {
  const { t } = useTranslation('client')

  if (entries.length === 0) return null

  return (
    <ul className={tokens.surface.divider}>
      {entries.map((entry) => (
        <li key={entry.id} className='flex items-center gap-3 py-2.5'>
          <IconTile tone={tone} className='size-7'>
            <Icon className='size-3.5' strokeWidth={1.75} />
          </IconTile>
          <div className='min-w-0 flex-1'>
            <div className='flex flex-wrap items-center gap-2'>
              <p className='text-xs font-medium text-neutral-900 dark:text-neutral-100'>{entry.label}</p>
              <Pill>{t('maintenance.inherited.badge')}</Pill>
            </div>
            {entry.sublabel && (
              <p className='mt-0.5 truncate text-xs text-neutral-500 dark:text-neutral-400'>{entry.sublabel}</p>
            )}
          </div>
          <span className='px-2 text-xs text-neutral-300 dark:text-neutral-600'>
            {t('maintenance.inherited.managed')}
          </span>
        </li>
      ))}
    </ul>
  )
}

export default function ClientMaintenanceTab({
  enabled,
  reason,
  strategy,
  whitelistCount,
  hasSettingsChanges,
  users,
  roles,
  whitelistedUserIds,
  whitelistedRoleIds,
  inheritedUsers,
  inheritedRoles,
  onToggle,
  onReasonChange,
  onStrategyChange,
  onWhitelistedUsersChange,
  onWhitelistedRolesChange,
  onSaveSettings,
  onResetSettings,
}: ClientMaintenanceTabProps) {
  const { t } = useTranslation('client')

  return (
    <>
      {enabled && (
        <div className='flex items-start gap-2.5 rounded-sm border border-fk-amber-border bg-fk-amber-soft/50 px-4 py-3'>
          <Wrench className='mt-0.5 size-4 shrink-0 text-fk-amber' strokeWidth={2} />
          <div className='min-w-0'>
            <p className='text-sm font-medium text-neutral-900 dark:text-neutral-100'>
              {t('maintenance.banner.title')}
            </p>
            <p className='mt-0.5 text-xs text-neutral-600 dark:text-neutral-400'>
              {t('maintenance.banner.detail', { count: whitelistCount })}
            </p>
          </div>
        </div>
      )}

      <Section title={t('maintenance.mode.title')} description={t('maintenance.mode.description')}>
        <FieldRow
          label={t('maintenance.mode.enabled.label')}
          description={t('maintenance.mode.enabled.description')}
        >
          <ChoiceCards
            label={t('maintenance.mode.state_picker')}
            value={maintenanceStateOf(enabled)}
            onChange={(v) => onToggle(isMaintenanceState(v))}
            options={maintenanceStateChoices(t)}
          />
        </FieldRow>

        <FieldRow
          label={t('maintenance.mode.reason.label')}
          description={t('maintenance.mode.reason.description')}
          htmlFor='maintenance-reason'
        >
          <Input
            id='maintenance-reason'
            value={reason}
            onChange={(e) => onReasonChange(e.target.value)}
            placeholder={t('maintenance.mode.reason.placeholder')}
            className='max-w-lg'
          />
        </FieldRow>

        <FieldRow
          label={t('maintenance.mode.strategy.label')}
          description={t('maintenance.mode.strategy.description')}
        >
          <ChoiceCards
            label={t('maintenance.mode.strategy.label')}
            value={strategy}
            onChange={onStrategyChange}
            options={sessionStrategyChoices(t)}
          />
        </FieldRow>
      </Section>

      <Section
        title={t('maintenance.users.title')}
        description={t('maintenance.users.description')}
        contained={inheritedUsers.length > 0}
      >
        <InheritedList entries={inheritedUsers} icon={User} tone='info' />
        <div className='py-3'>
          <EntityPicker
            items={users}
            value={whitelistedUserIds}
            onChange={onWhitelistedUsersChange}
            addLabel={t('maintenance.users.add')}
            searchPlaceholder={t('maintenance.users.search_placeholder')}
            emptyIcon={User}
            emptyHint={t('maintenance.users.empty')}
            exhaustedHint={t('maintenance.users.exhausted')}
          />
        </div>
      </Section>

      <Section
        title={t('maintenance.roles.title')}
        description={t('maintenance.roles.description')}
        contained={inheritedRoles.length > 0}
      >
        <InheritedList entries={inheritedRoles} icon={ShieldCheck} tone='violet' />
        <div className='py-3'>
          <EntityPicker
            items={roles}
            value={whitelistedRoleIds}
            onChange={onWhitelistedRolesChange}
            addLabel={t('maintenance.roles.add')}
            searchPlaceholder={t('maintenance.roles.search_placeholder')}
            emptyIcon={ShieldCheck}
            emptyHint={t('maintenance.roles.empty')}
            exhaustedHint={t('maintenance.roles.exhausted')}
          />
        </div>
      </Section>

      <SaveBar
        show={hasSettingsChanges}
        title={t('maintenance.save.title')}
        description={t('maintenance.save.description')}
        onCancel={onResetSettings}
        cancelLabel={t('shared.discard')}
        actions={[{ label: t('shared.save'), onClick: onSaveSettings }]}
      />
    </>
  )
}
