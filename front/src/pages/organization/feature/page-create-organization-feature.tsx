import { useCreateOrganization } from '@/api/organization.api'
import { zodResolver } from '@hookform/resolvers/zod'
import { RouterParams } from '@/routes/router'
import {
  ORGANIZATIONS_URL,
  ORGANIZATION_OVERVIEW_URL,
  ORGANIZATION_SETTINGS_URL,
  ORGANIZATION_URL,
} from '@/routes/sub-router/organization.router'
import { useForm } from 'react-hook-form'
import { useNavigate, useParams } from 'react-router'
import {
  CreateOrganizationSchema,
  createOrganizationSchema,
} from '../schemas/create-organization.schema'
import { Form } from '@/components/ui/form'
import PageCreateOrganization from '../ui/page-create-organization'
import { toast } from 'sonner'

export default function PageCreateOrganizationFeature() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const { mutate: createOrganization } = useCreateOrganization()

  const form = useForm<CreateOrganizationSchema>({
    resolver: zodResolver(createOrganizationSchema),
    mode: 'onChange',
    defaultValues: {
      name: '',
      alias: '',
      domain: '',
      redirectUrl: '',
      description: '',
      enabled: true,
    },
  })

  const handleBack = () => {
    navigate(`${ORGANIZATIONS_URL(realm_name)}${ORGANIZATION_OVERVIEW_URL}`)
  }

  const handleSubmit = () => {
    const values = form.getValues()
    if (!realm_name) return

    createOrganization(
      {
        path: { realm_name },
        body: {
          name: values.name,
          alias: values.alias,
          domain: values.domain || null,
          redirect_url: values.redirectUrl || null,
          description: values.description || null,
          enabled: values.enabled,
        },
      },
      {
        onSuccess: (payload) => {
          navigate(`${ORGANIZATION_URL(realm_name, payload.id)}${ORGANIZATION_SETTINGS_URL}`)
        },
        onError: () => {
          toast.error('Failed to create organization')
        },
      }
    )
  }

  const formIsValid = form.formState.isValid

  return (
    <Form {...form}>
      <PageCreateOrganization
        form={form}
        handleBack={handleBack}
        handleSubmit={handleSubmit}
        formIsValid={formIsValid}
      />
    </Form>
  )
}
