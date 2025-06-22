import { useEffect, useState } from 'react'
import RoleMappingModal from '../../ui/modals/role-mapping-modal'
import { useGetRoles } from '@/api/role.api'
import { useParams } from 'react-router'
import { RouterParams } from '@/routes/router'
import { useAssignUserRole, useGetUser, useGetUserRoles } from '@/api/user.api'
import { useForm } from 'react-hook-form'
import { assignRoleSchema, AssignRoleSchema } from '../../schemas/assign-role.schema'
import { zodResolver } from '@hookform/resolvers/zod'
import { Form } from '@/components/ui/form'
import { Role } from '@/api/api.interface'
import { toast } from 'sonner'

export default function RoleMappingModalFeature() {
  const { realm_name, user_id } = useParams<RouterParams>()
  const [open, setOpen] = useState(false)
  const [availableRoles, setAvailableRoles] = useState<Role[]>([])

  const { mutate: assignRole, data } = useAssignUserRole()
  const { data: rolesResponse } = useGetRoles({ realm: realm_name })
  const { data: user } = useGetUser({
    realm: realm_name,
    userId: user_id,
  })
  const { data: userRoles } = useGetUserRoles({
    realm: realm_name,
    userId: user_id || '',
  })

  const form = useForm<AssignRoleSchema>({
    resolver: zodResolver(assignRoleSchema),
    mode: 'onChange',
    defaultValues: {
      roleIds: [],
    },
  })

  useEffect(() => {
    if (userRoles && rolesResponse) {
      const allRoles = rolesResponse.data
      const assignedRoleIds = userRoles.data.map((role) => role.id)

      const unassignedRoles = allRoles.filter((role) => !assignedRoleIds.includes(role.id))
      setAvailableRoles(unassignedRoles)
    }
  }, [userRoles, rolesResponse])

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
    form.reset()
    setOpen(false)
  })

  const isValid = form.formState.isValid

  useEffect(() => {
    if (data) {
      toast.success('Role(s) assigned successfully')
    }
  }, [data])

  if (!user) {
    return null
  }

  return (
    <Form {...form}>
      <RoleMappingModal
        open={open}
        setOpen={setOpen}
        roles={availableRoles}
        user={user}
        form={form}
        isValid={isValid}
        handleSubmit={handleSubmit}
      />
    </Form>
  )
}
