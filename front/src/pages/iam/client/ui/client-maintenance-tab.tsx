import { ShieldCheck, User, Wrench } from 'lucide-react'
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
import { maintenanceStateChoices, sessionStrategyChoices } from '../client-choices'

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
  kind,
}: {
  entries: InheritedEntry[]
  kind: 'user' | 'role'
}) {
  if (entries.length === 0) return null

  return (
    <ul className={tokens.surface.divider}>
      {entries.map((entry) => (
        <li key={entry.id} className='flex items-center gap-3 py-2.5'>
          <IconTile tone={kind === 'user' ? 'info' : 'violet'} className='size-7'>
            {kind === 'user' ? (
              <User className='size-3.5' strokeWidth={1.75} />
            ) : (
              <ShieldCheck className='size-3.5' strokeWidth={1.75} />
            )}
          </IconTile>
          <div className='min-w-0 flex-1'>
            <div className='flex flex-wrap items-center gap-2'>
              <p className='text-xs font-medium text-neutral-900 dark:text-neutral-100'>{entry.label}</p>
              <Pill>inherited from the realm</Pill>
            </div>
            {entry.sublabel && (
              <p className='mt-0.5 truncate text-xs text-neutral-500 dark:text-neutral-400'>{entry.sublabel}</p>
            )}
          </div>
          <span className='px-2 text-xs text-neutral-300 dark:text-neutral-600'>managed at the realm</span>
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
  return (
    <>
      {enabled && (
        <div className='flex items-start gap-2.5 rounded-sm border border-fk-amber-border bg-fk-amber-soft/50 px-4 py-3'>
          <Wrench className='mt-0.5 size-4 shrink-0 text-fk-amber' strokeWidth={2} />
          <div className='min-w-0'>
            <p className='text-sm font-medium text-neutral-900 dark:text-neutral-100'>This client is in maintenance</p>
            <p className='mt-0.5 text-xs text-neutral-600 dark:text-neutral-400'>
              {whitelistCount} whitelist entr{whitelistCount === 1 ? 'y' : 'ies'} can still
              authenticate. Every other request is refused.
            </p>
          </div>
        </div>
      )}

      <Section
        title='Maintenance mode'
        description='Temporarily restricts access without touching the client configuration.'
      >
        <FieldRow
          label='Maintenance enabled'
          description='Once active, only the whitelisted users and roles can authenticate. Applied as soon as you switch it.'
        >
          <ChoiceCards
            label='Maintenance state'
            value={enabled ? 'maintenance' : 'open'}
            onChange={(v) => onToggle(v === 'maintenance')}
            options={maintenanceStateChoices}
          />
        </FieldRow>

        <FieldRow
          label='Reason'
          description='Message displayed to blocked users on the login page.'
          htmlFor='maintenance-reason'
        >
          <Input
            id='maintenance-reason'
            value={reason}
            onChange={(e) => onReasonChange(e.target.value)}
            placeholder='Migration under way, back at 2pm.'
            className='max-w-lg'
          />
        </FieldRow>

        <FieldRow
          label='Session strategy'
          description='What happens to sessions already open when maintenance is switched on.'
        >
          <ChoiceCards
            label='Session strategy'
            value={strategy}
            onChange={onStrategyChange}
            options={sessionStrategyChoices}
          />
        </FieldRow>
      </Section>

      <Section
        title='Allowed users'
        description='Accounts named individually as allowed during maintenance. Realm-level entries are inherited automatically.'
        contained={inheritedUsers.length > 0}
      >
        <InheritedList entries={inheritedUsers} kind='user' />
        <div className='py-3'>
          <EntityPicker
            items={users}
            value={whitelistedUserIds}
            onChange={onWhitelistedUsersChange}
            addLabel='Add a user'
            searchPlaceholder='Search users…'
            emptyIcon={User}
            emptyHint='No user allowed on this client.'
            exhaustedHint='Every user of the realm is already allowed.'
          />
        </div>
      </Section>

      <Section
        title='Allowed roles'
        description='Every holder of these roles is allowed during maintenance. Realm-level entries are inherited automatically.'
        contained={inheritedRoles.length > 0}
      >
        <InheritedList entries={inheritedRoles} kind='role' />
        <div className='py-3'>
          <EntityPicker
            items={roles}
            value={whitelistedRoleIds}
            onChange={onWhitelistedRolesChange}
            addLabel='Add a role'
            searchPlaceholder='Search roles…'
            emptyIcon={ShieldCheck}
            emptyHint='No role allowed on this client.'
            exhaustedHint='Every role of the realm is already allowed.'
          />
        </div>
      </Section>

      <SaveBar
        show={hasSettingsChanges}
        title='Unsaved maintenance settings'
        description='Save the maintenance reason and session strategy changes.'
        onCancel={onResetSettings}
        cancelLabel='Discard'
        actions={[{ label: 'Save changes', onClick: onSaveSettings }]}
      />
    </>
  )
}
