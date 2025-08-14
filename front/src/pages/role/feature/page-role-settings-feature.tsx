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

  const { data: roleResponse, isLoading } = useGetRole({
    realm: realm_name || 'master',
    roleId: role_id,
  })
  const { mutate: udpateRole } = useUpdateRole()

  const form = useForm<UpdateRoleSchema>({
    resolver: zodResolver(updateRoleSchema),
    mode: 'onChange',
    defaultValues: {
      name: roleResponse?.data.name || '',
      description: roleResponse?.data.description || '',
    },
  })

  const hasChanges = useFormChanges(
    form,
    roleResponse && {
      name: roleResponse.data.name || '',
      description: roleResponse.data.description || '',
    }
  )

  const handleSubmit = form.handleSubmit((values) => {
    if (!realm_name || !role_id) return

    udpateRole({
      body: {
        description: values.description,
        name: values.name
      },
      path: {
        realm_name: realm_name,
        role_id: role_id
      }
    })
  })

  useEffect(() => {
    if (roleResponse) {
      form.reset({
        name: roleResponse.data.name,
        description: roleResponse.data.description || '',
      })
    }
  }, [roleResponse])

  return (
    <Form {...form}>
      <PageRoleSettings
        role={roleResponse?.data}
        form={form}
        isLoading={isLoading}
        realmName={realm_name || 'master'}
        hasChanges={hasChanges}
        handleSubmit={handleSubmit}
      />
    </Form>
  )
}
