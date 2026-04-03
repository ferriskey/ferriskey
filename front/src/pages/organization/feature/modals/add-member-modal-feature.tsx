import { useEffect, useMemo, useState } from 'react'
import AddMemberModal from '../../ui/modals/add-member-modal'
import {
  useAddUserToOrganization,
  useGetOrganization,
  useGetOrganizationMembers,
} from '@/api/organization.api'
import { useGetUsers } from '@/api/user.api'
import { useParams } from 'react-router'
import { RouterParams } from '@/routes/router'
import { useForm } from 'react-hook-form'
import { addMemberSchema, AddMemberSchema } from '../../schemas/add-member.schema'
import { zodResolver } from '@hookform/resolvers/zod'
import { Form } from '@/components/ui/form'
import { toast } from 'sonner'
import { Schemas } from '@/api/api.client'
import User = Schemas.User

export default function AddMemberModalFeature() {
  const { realm_name, organizationId } = useParams<RouterParams & { organizationId: string }>()
  const [open, setOpen] = useState(false)

  const { mutate: addMember, data } = useAddUserToOrganization()
  const { data: orgResponse } = useGetOrganization({ realm: realm_name, organizationId })
  const { data: usersResponse } = useGetUsers({ realm: realm_name })
  const { data: members } = useGetOrganizationMembers({ realm: realm_name, organizationId })

  const form = useForm<AddMemberSchema>({
    resolver: zodResolver(addMemberSchema),
    mode: 'onChange',
    defaultValues: { userIds: [] },
  })

  const availableUsers = useMemo<User[]>(() => {
    if (!usersResponse || !members) return []
    const memberIds = new Set(members.map((m) => m.user_id))
    return usersResponse.data.filter((u) => !memberIds.has(u.id))
  }, [usersResponse, members])

  const handleSubmit = form.handleSubmit((values) => {
    if (!organizationId || !realm_name) {
      toast.error('Organization or realm not found')
      return
    }
    for (const userId of values.userIds) {
      addMember({
        path: { realm_name, organization_id: organizationId },
        body: { user_id: userId },
      })
    }
    form.reset()
    setOpen(false)
  })

  useEffect(() => {
    if (data) {
      toast.success('Member(s) added successfully')
    }
  }, [data])

  if (!orgResponse) return null

  return (
    <Form {...form}>
      <AddMemberModal
        open={open}
        setOpen={setOpen}
        users={availableUsers}
        organization={orgResponse}
        form={form}
        isValid={form.formState.isValid}
        handleSubmit={handleSubmit}
      />
    </Form>
  )
}
