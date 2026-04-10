import { useEffect, useMemo, useState } from 'react'
import AssignOrganizationModal from '../../ui/modals/assign-organization-modal'
import { useGetOrganizations, useAddUserToOrganization, useGetUserOrganizations } from '@/api/organization.api'
import { useParams } from 'react-router'
import { RouterParams } from '@/routes/router'
import { useGetUser } from '@/api/user.api'
import { useForm } from 'react-hook-form'
import { assignOrganizationSchema, AssignOrganizationSchema } from '../../schemas/assign-organization.schema'
import { zodResolver } from '@hookform/resolvers/zod'
import { Form } from '@/components/ui/form'
import { toast } from 'sonner'
import { Schemas } from '@/api/api.client'
import Organization = Schemas.Organization

export default function AssignOrganizationModalFeature() {
  const { realm_name, user_id } = useParams<RouterParams>()
  const [open, setOpen] = useState(false)

  const { mutate: addToOrg, data } = useAddUserToOrganization()
  const { data: orgsResponse } = useGetOrganizations({ realm: realm_name })
  const { data: userResponse } = useGetUser({ realm: realm_name, userId: user_id })
  const { data: userOrgs } = useGetUserOrganizations({
    realm: realm_name,
    userId: user_id,
  })

  const form = useForm<AssignOrganizationSchema>({
    resolver: zodResolver(assignOrganizationSchema),
    mode: 'onChange',
    defaultValues: {
      organizationIds: [],
    },
  })

  const availableOrganizations = useMemo<Organization[]>(() => {
    if (!orgsResponse || !userOrgs) return []
    const allOrgs = orgsResponse.data
    const assignedOrgIds = new Set(userOrgs.map((m) => m.organization_id))
    return allOrgs.filter((org) => !assignedOrgIds.has(org.id))
  }, [orgsResponse, userOrgs])

  const handleSubmit = form.handleSubmit((values) => {
    if (!user_id || !realm_name) {
      toast.error('User or realm not found')
      return
    }

    for (const orgId of values.organizationIds) {
      addToOrg({
        path: { realm_name, organization_id: orgId },
        body: { user_id },
      })
    }
    form.reset()
    setOpen(false)
  })

  const isValid = form.formState.isValid

  useEffect(() => {
    if (data) {
      toast.success('Organization(s) assigned successfully')
    }
  }, [data])

  if (!userResponse) {
    return null
  }

  return (
    <Form {...form}>
      <AssignOrganizationModal
        open={open}
        setOpen={setOpen}
        organizations={availableOrganizations}
        user={userResponse.data}
        form={form}
        isValid={isValid}
        handleSubmit={handleSubmit}
      />
    </Form>
  )
}
