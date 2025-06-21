import { useEffect, useState } from 'react'
import RoleMappingModal from '../../ui/modals/role-mapping-modal'
import { useGetRoles } from '@/api/role.api'
import { useParams } from 'react-router'
import { RouterParams } from '@/routes/router'
import { useAssignUserRole, useGetUser } from '@/api/user.api'
import { useForm, useWatch } from 'react-hook-form'
import { assignRoleSchema, AssignRoleSchema } from '../../schemas/assign-role.schema'
import { zodResolver } from '@hookform/resolvers/zod'
import { Form } from '@/components/ui/form'

export default function RoleMappingModalFeature() {
  const { realm_name, user_id } = useParams<RouterParams>()
  const [open, setOpen] = useState(false)
  const { mutate: assignRole } = useAssignUserRole()
  const { data: rolesResponse } = useGetRoles({ realm: realm_name })
  const { data: user } = useGetUser({
    realm: realm_name,
    userId: user_id,
  })

  const form = useForm<AssignRoleSchema>({
    resolver: zodResolver(assignRoleSchema),
    mode: 'onChange',
    defaultValues: {
      roleIds: [],
    },
  })

  if (!user) {
    return null // or handle loading state
  }

  const handleSubmit = form.handleSubmit((values) => {
    for (const roleId of values.roleIds) {
      assignRole({
        realm: realm_name,
        userId: user_id,
        payload: {
          roleId,
        },
      })
    }
  })

  const isValid = form.formState.isValid

  return (
    <Form {...form}>
      <RoleMappingModal
        open={open}
        setOpen={setOpen}
        roles={rolesResponse?.data ?? []}
        user={user}
        form={form}
        isValid={isValid}
        handleSubmit={handleSubmit}
      />
    </Form>
  )
}
