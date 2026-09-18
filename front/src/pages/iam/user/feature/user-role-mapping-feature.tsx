import { useMemo, useState } from 'react'
import { useParams } from 'react-router'
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'
import { useGetUserRoles } from '@/api/user.api'
import { useAssignUserRole, useUnassignUserRole } from '@/api/user_role.api'
import { useGetRoles } from '@/api/role.api'
import { RouterParams } from '@/routes/router'
import { assignRoleSchema } from '@/pages/iam/user/schemas/assign-role.schema'
import UserRoleMappingTab from '../ui/user-role-mapping-tab'

export default function UserRoleMappingFeature() {
  const { realm_name, user_id } = useParams<RouterParams>()
  const { t } = useTranslation('user')
  const realm = realm_name ?? 'master'

  const {
    data: userRoles,
    isLoading,
    isError,
  } = useGetUserRoles({ realm, userId: user_id ?? '' })
  const { data: rolesResponse } = useGetRoles({ realm })
  const { mutate: assignRole } = useAssignUserRole()
  const { mutate: unassignRole } = useUnassignUserRole()

  const [selectedRoleIds, setSelectedRoleIds] = useState<string[]>([])

  const roles = useMemo(() => userRoles?.data ?? [], [userRoles])

  const assignableRoles = useMemo(() => {
    const assigned = new Set(roles.map((role) => role.id))
    return (rolesResponse?.data ?? []).filter((role) => !assigned.has(role.id))
  }, [roles, rolesResponse])

  const handleAssign = () => {
    if (!user_id || !realm_name) {
      toast.error(t('detail.role_mapping.toast.missing_context'))
      return
    }
    if (!assignRoleSchema.safeParse({ roleIds: selectedRoleIds }).success) return

    const assignedCount = selectedRoleIds.length

    for (const roleId of selectedRoleIds) {
      assignRole({ path: { realm_name, user_id, role_id: roleId } })
    }
    setSelectedRoleIds([])
    toast.success(t('detail.role_mapping.toast.assigned', { count: assignedCount }))
  }

  const handleUnassign = (roleId: string) => {
    if (!user_id || !realm_name) return
    unassignRole(
      { path: { realm_name, user_id, role_id: roleId } },
      {
        onSuccess: () =>
          toast.success(t('detail.role_mapping.toast.unassigned_title'), {
            description: t('detail.role_mapping.toast.unassigned_description'),
          }),
      }
    )
  }

  return (
    <UserRoleMappingTab
      roles={roles}
      assignableRoles={assignableRoles}
      isLoading={isLoading}
      isError={isError}
      selectedRoleIds={selectedRoleIds}
      onSelectedRoleIdsChange={setSelectedRoleIds}
      onAssign={handleAssign}
      onUnassign={handleUnassign}
    />
  )
}
