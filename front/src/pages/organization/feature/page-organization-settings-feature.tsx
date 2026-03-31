import { useDeleteOrganization, useGetOrganization, useUpdateOrganization } from '@/api/organization.api'
import { useFormChanges } from '@/hooks/use-form-changes'
import { RouterParams } from '@/routes/router'
import { ORGANIZATIONS_URL } from '@/routes/sub-router/organization.router'
import { zodResolver } from '@hookform/resolvers/zod'
import { useEffect } from 'react'
import { useForm } from 'react-hook-form'
import { useNavigate, useParams } from 'react-router'
import { Form } from '@/components/ui/form'
import {
  updateOrganizationSchema,
  UpdateOrganizationSchema,
} from '../schemas/update-organization.schema'
import PageOrganizationSettings from '../ui/page-organization-settings'

export default function PageOrganizationSettingsFeature() {
  const { realm_name, organizationId } = useParams<RouterParams & { organizationId: string }>()
  const navigate = useNavigate()

  const { data: orgResponse, isLoading } = useGetOrganization({
    realm: realm_name,
    organizationId,
  })

  const { mutate: updateOrganization } = useUpdateOrganization()
  const { mutateAsync: deleteOrganization } = useDeleteOrganization()

  const form = useForm<UpdateOrganizationSchema>({
    resolver: zodResolver(updateOrganizationSchema),
    defaultValues: {
      name: '',
      alias: '',
      domain: null,
      redirectUrl: null,
      description: null,
      enabled: true,
    },
  })

  const hasChanges = useFormChanges(
    form,
    orgResponse && {
      name: orgResponse.name,
      alias: orgResponse.alias,
      domain: orgResponse.domain ?? null,
      redirectUrl: orgResponse.redirect_url ?? null,
      description: orgResponse.description ?? null,
      enabled: orgResponse.enabled,
    }
  )

  useEffect(() => {
    if (orgResponse) {
      form.reset({
        name: orgResponse.name,
        alias: orgResponse.alias,
        domain: orgResponse.domain ?? null,
        redirectUrl: orgResponse.redirect_url ?? null,
        description: orgResponse.description ?? null,
        enabled: orgResponse.enabled,
      })
    }
  }, [orgResponse, form])

  const handleSubmit = form.handleSubmit((values) => {
    if (!orgResponse || !realm_name) return
    updateOrganization({
      body: {
        name: values.name,
        alias: values.alias,
        domain: values.domain ?? null,
        redirect_url: values.redirectUrl ?? null,
        description: values.description ?? null,
        enabled: values.enabled,
      },
      path: {
        realm_name,
        organization_id: orgResponse.id,
      },
    })
  })

  const handleDelete = async () => {
    if (!orgResponse || !realm_name) return
    try {
      await deleteOrganization({
        path: { realm_name, organization_id: orgResponse.id },
      })
      navigate(`${ORGANIZATIONS_URL(realm_name)}/overview`)
    } catch {
      // error handled by the mutation hook
    }
  }

  if (isLoading) {
    return (
      <div className='flex flex-col gap-6 py-8'>
        {[1, 2, 3].map((i) => (
          <div key={i} className='flex items-start justify-between py-4 border-t animate-pulse'>
            <div className='w-1/3 space-y-2'>
              <div className='h-4 bg-muted rounded w-1/2' />
              <div className='h-3 bg-muted rounded w-3/4' />
            </div>
            <div className='w-1/2 h-9 bg-muted rounded' />
          </div>
        ))}
      </div>
    )
  }

  if (!orgResponse) return null

  return (
    <Form {...form}>
      <PageOrganizationSettings
        organization={orgResponse}
        form={form}
        handleSubmit={handleSubmit}
        hasChanges={hasChanges}
        onDelete={handleDelete}
      />
    </Form>
  )
}
