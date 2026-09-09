import { useMemo, useState } from 'react'
import { useNavigate, useParams } from 'react-router'
import {
  useDeleteRole,
  useGetRole,
  useUpdateRole,
  useUpdateRolePermissions,
} from '@/api/role.api'
import { RouterParams } from '@/routes/router'
import { useRouteTabs } from '@/components/kit'
import { updateRoleSchema } from '@/pages/role/schemas/update-role.schema'
import PageRoleDetail from '@/next/pages/iam/role/ui/page-role-detail'
import { CONSOLE_ROLE_URL, CONSOLE_ROLES_URL } from '../urls'

interface Draft {
  key: string
  name: string
  description: string
  permissions: string[]
}

const EMPTY_DRAFT: Draft = { key: '', name: '', description: '', permissions: [] }

const ROLE_TABS = [
  { key: 'settings', label: 'Settings' },
  { key: 'permissions', label: 'Permissions' },
  { key: 'users', label: 'Identities' },
] as const

export default function PageRoleDetailFeature() {
  const { realm_name, role_id } = useParams<RouterParams>()
  const navigate = useNavigate()
  const realm = realm_name ?? 'master'

  const { data: roleResponse, isLoading } = useGetRole({ realm, roleId: role_id })
  const { mutate: updateRole } = useUpdateRole()
  const { mutate: updateRolePermissions } = useUpdateRolePermissions()
  const { mutateAsync: deleteRole } = useDeleteRole()

  const role = roleResponse?.data
  const { value: tab, tabs } = useRouteTabs(CONSOLE_ROLE_URL(realm, role_id), ROLE_TABS)

  const [draft, setDraft] = useState<Draft>(EMPTY_DRAFT)

  const roleKey = role?.id ?? ''
  const pristine: Draft = role
    ? {
        key: roleKey,
        name: role.name,
        description: role.description ?? '',
        permissions: role.permissions,
      }
    : EMPTY_DRAFT

  if (role && draft.key !== roleKey) setDraft(pristine)

  const { name, description, permissions } = draft.key === roleKey ? draft : pristine

  const nameError = useMemo(() => {
    const parsed = updateRoleSchema.safeParse({ name, description })
    if (parsed.success) return undefined
    return parsed.error.issues.find((issue) => issue.path[0] === 'name')?.message
  }, [name, description])

  const definitionDirty = Boolean(
    role && (name !== role.name || description !== (role.description ?? ''))
  )
  const permissionsDirty = Boolean(
    role &&
      (permissions.length !== role.permissions.length ||
        permissions.some((permission) => !role.permissions.includes(permission)))
  )
  const dirtyCount = (definitionDirty ? 1 : 0) + (permissionsDirty ? 1 : 0)

  const save = () => {
    if (!role || !role_id || nameError) return

    if (definitionDirty) {
      updateRole({
        body: { name, description },
        path: { realm_name: realm, role_id },
      })
    }

    if (permissionsDirty) {
      updateRolePermissions({
        body: { permissions },
        path: { realm_name: realm, role_id },
      })
    }
  }

  const handleDelete = async () => {
    if (!role_id) return
    await deleteRole({ path: { realm_name: realm, role_id } })
      .then(() => navigate(CONSOLE_ROLES_URL(realm)))
      .catch(() => undefined)
  }

  return (
    <PageRoleDetail
      role={role}
      isLoading={isLoading}
      tab={tab}
      tabs={tabs}
      name={name}
      description={description}
      permissions={permissions}
      nameError={definitionDirty ? nameError : undefined}
      dirtyCount={dirtyCount}
      onNameChange={(v) => setDraft((d) => ({ ...d, name: v }))}
      onDescriptionChange={(v) => setDraft((d) => ({ ...d, description: v }))}
      onPermissionsChange={(v) => setDraft((d) => ({ ...d, permissions: v }))}
      onBack={() => navigate(CONSOLE_ROLES_URL(realm))}
      onDiscard={() => setDraft(pristine)}
      onSave={save}
      onDelete={handleDelete}
    />
  )
}
