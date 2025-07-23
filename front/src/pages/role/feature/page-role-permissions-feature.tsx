import { useGetRole, useUpdateRolePermissions } from '@/api/role.api.ts'
import PageRolePermissions from '@/pages/role/ui/page-role-permissions.tsx'
import { RouterParams } from '@/routes/router.ts'
import { zodResolver } from '@hookform/resolvers/zod'
import { useForm } from 'react-hook-form'
import { useParams } from 'react-router'
import { Form } from '../../../components/ui/form'
import { updateRolePermissionsSchema, UpdateRolePermissionsSchema } from '../schemas/update-role.schema'

export default function PageRolePermissionsFeature() {
  const { realm_name, role_id } = useParams<RouterParams>()
  const { mutate: updatePermissions } = useUpdateRolePermissions()

  const { data: role } = useGetRole({
    realm: realm_name || 'master',
    roleId: role_id,
  })

  const form = useForm<UpdateRolePermissionsSchema>({
    resolver: zodResolver(updateRolePermissionsSchema),
    values: {
      permissions: role?.permissions ?? [],
    },
  })

  function togglePermission(permission: string) {
    const currentPermissions = form.getValues('permissions') || [];
    const copy = currentPermissions.includes(permission)
      ? currentPermissions.filter((p) => p !== permission)
      : [...currentPermissions, permission];

    form.setValue('permissions', copy, { shouldDirty: true });
  }

  function handleSubmit() {
    updatePermissions({
      realmName: realm_name || 'master',
      roleId: role_id!,
      payload: form.getValues(),
    })
  }

  if (!role) return null

  return (
    <Form {...form}>
      <PageRolePermissions
        role={role}
        togglePermission={togglePermission}
        handleSubmit={handleSubmit}
      />
    </Form>
  )
}
