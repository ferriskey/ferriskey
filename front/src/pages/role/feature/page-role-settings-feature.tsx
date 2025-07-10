import { useParams } from 'react-router-dom'
import { useGetRole, useUpdateRole } from '@/api/role.api'
import PageRoleSettings from '../ui/page-role-settings'
import { RouterParams } from '@/routes/router'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { UpdateRoleSchema, updateRoleSchema } from '@/pages/role/schemas/update-role.schema.ts'
import { useEffect } from 'react'
import { Form } from '@/components/ui/form.tsx'
import { useFormChanges } from '@/hooks/use-form-changes'

export default function PageRoleSettingsFeature() {
  const { realm_name, role_id } = useParams<RouterParams>()

  const { data: role, isLoading } = useGetRole({
    realm: realm_name || 'master',
    roleId: role_id,
  })
  const { mutate: udpateRole } = useUpdateRole()

  const form = useForm<UpdateRoleSchema>({
    resolver: zodResolver(updateRoleSchema),
    mode: 'onChange',
    defaultValues: {
      name: role?.name || '',
      description: role?.description || '',
    },
  })

  const hasChanges = useFormChanges(
    form,
    role && {
      name: role.name || '',
      description: role.description || '',
    }
  )

  const handleSubmit = form.handleSubmit((values) => {
    udpateRole({
      payload: values,
      realmName: realm_name || 'master',
      roleId: role_id || '',
    })
  })

  useEffect(() => {
    if (role) {
      form.reset({
        name: role.name,
        description: role.description || '',
      })
    }
  }, [role])

  return (
    <Form {...form}>
      <PageRoleSettings
        role={role}
        form={form}
        isLoading={isLoading}
        realmName={realm_name || 'master'}
        hasChanges={hasChanges}
        handleSubmit={handleSubmit}
      />
    </Form>
  )
}
