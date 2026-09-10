import { EntityPicker, FieldRow, Section } from '@/components/kit'
import type { PickableEntity } from '@/components/kit'

export interface RealmMaintenanceTabProps {
  users: PickableEntity[]
  roles: PickableEntity[]
  selectedUserIds: string[]
  selectedRoleIds: string[]
  onUsersChange: (next: string[]) => void
  onRolesChange: (next: string[]) => void
}

export default function RealmMaintenanceTab({
  users,
  roles,
  selectedUserIds,
  selectedRoleIds,
  onUsersChange,
  onRolesChange,
}: RealmMaintenanceTabProps) {
  return (
    <>
      <div className='rounded-lg border border-fk-info-border bg-fk-info-soft/40 px-4 py-3 text-xs text-neutral-700 dark:text-neutral-300'>
        These entries add to the whitelist of each client: they allow authentication on
        every client of this realm placed under maintenance.
      </div>

      <Section
        title='Default maintenance members'
        description='Accounts and roles always allowed during a maintenance of this realm.'
      >
        <FieldRow
          layout='stacked'
          label='Users'
          description='Individual accounts always allowed during maintenance.'
        >
          <EntityPicker
            fullWidth
            items={users}
            value={selectedUserIds}
            onChange={onUsersChange}
            addLabel='Add a user'
            searchPlaceholder='Search users…'
            emptyHint='No users available.'
            exhaustedHint='Every account of this realm is already allowed.'
          />
        </FieldRow>

        <FieldRow
          layout='stacked'
          className='border-t border-fk-line pt-4'
          label='Roles'
          description='Everyone holding one of these roles is allowed during maintenance.'
        >
          <EntityPicker
            fullWidth
            items={roles}
            value={selectedRoleIds}
            onChange={onRolesChange}
            addLabel='Add a role'
            searchPlaceholder='Search roles…'
            emptyHint='No roles available.'
            exhaustedHint='Every role of this realm is already allowed.'
          />
        </FieldRow>
      </Section>
    </>
  )
}
