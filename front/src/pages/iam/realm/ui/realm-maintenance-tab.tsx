import { useTranslation } from 'react-i18next'
import { EntityPicker, FieldRow, Section } from '@/components/kit'
import type { PickableEntity } from '@/components/kit'
import { REALM_NAMESPACE } from '../realm-namespace'

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
  const { t } = useTranslation(REALM_NAMESPACE)

  return (
    <>
      <div className='rounded-lg border border-fk-info-border bg-fk-info-soft/40 px-4 py-3 text-xs text-neutral-700 dark:text-neutral-300'>
        {t('maintenance.notice')}
      </div>

      <Section
        title={t('maintenance.title')}
        description={t('maintenance.description')}
        divided={false}
      >
        <FieldRow
          layout='stacked'
          label={t('maintenance.users.label')}
          description={t('maintenance.users.description')}
        >
          <EntityPicker
            fullWidth
            items={users}
            value={selectedUserIds}
            onChange={onUsersChange}
            addLabel={t('maintenance.users.add')}
            searchPlaceholder={t('maintenance.users.search_placeholder')}
            emptyHint={t('maintenance.users.empty_hint')}
            exhaustedHint={t('maintenance.users.exhausted_hint')}
          />
        </FieldRow>

        <FieldRow
          layout='stacked'
          label={t('maintenance.roles.label')}
          description={t('maintenance.roles.description')}
        >
          <EntityPicker
            fullWidth
            items={roles}
            value={selectedRoleIds}
            onChange={onRolesChange}
            addLabel={t('maintenance.roles.add')}
            searchPlaceholder={t('maintenance.roles.search_placeholder')}
            emptyHint={t('maintenance.roles.empty_hint')}
            exhaustedHint={t('maintenance.roles.exhausted_hint')}
          />
        </FieldRow>
      </Section>
    </>
  )
}
