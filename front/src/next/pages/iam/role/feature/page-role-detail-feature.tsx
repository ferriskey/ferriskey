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
import { NEXT_ROLE_URL, NEXT_ROLES_URL } from '@/next/routes'
import PageRoleDetail from '../ui/page-role-detail'
import { useCrumbLabel } from '@/next/shell/crumb-store'

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
  { key: 'users', label: 'Users' },
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
  const { value: tab, tabs } = useRouteTabs(NEXT_ROLE_URL(realm, role_id), ROLE_TABS)

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

  const setName = (v: string) => setDraft((d) => ({ ...d, name: v }))
  const setDescription = (v: string) => setDraft((d) => ({ ...d, description: v }))
  const setPermissions = (v: string[]) => setDraft((d) => ({ ...d, permissions: v }))

  const nameError = useMemo(() => {
    const parsed = updateRoleSchema.safeParse({ name, description })
    if (parsed.success) return undefined
    return parsed.error.issues.find((i) => i.path[0] === 'name')?.message
  }, [name, description])

  const definitionDirty = Boolean(
    role && (name !== role.name || description !== (role.description ?? ''))
  )
  const permissionsDirty = Boolean(
    role &&
      (permissions.length !== role.permissions.length ||
        permissions.some((p) => !role.permissions.includes(p)))
  )
  const dirtyCount = (definitionDirty ? 1 : 0) + (permissionsDirty ? 1 : 0)

  const reset = () => setDraft(pristine)

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
      .then(() => navigate(NEXT_ROLES_URL(realm)))
      .catch(() => undefined)
  }


  useCrumbLabel(role_id, role?.name)

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
      onNameChange={setName}
      onDescriptionChange={setDescription}
      onPermissionsChange={setPermissions}
      onBack={() => navigate(NEXT_ROLES_URL(realm))}
      onDiscard={reset}
      onSave={save}
      onDelete={handleDelete}
    />
  )
}
