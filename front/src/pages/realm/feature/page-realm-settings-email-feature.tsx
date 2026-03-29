import { useGetEmailTemplates, useDeleteEmailTemplate } from '@/api/email-template.api'
import { useGetRealm, useUpdateRealmSettings } from '@/api/realm.api'
import { useDeleteSmtpConfig, useGetSmtpConfig, useUpsertSmtpConfig } from '@/api/smtp.api'
import { RouterParams } from '@/routes/router'
import { zodResolver } from '@hookform/resolvers/zod'
import { useForm } from 'react-hook-form'
import { useNavigate, useParams } from 'react-router'
import { EMAIL_TEMPLATE_BUILDER_URL } from '@/routes/sub-router/email-template.router'
import { z } from 'zod'
import PageRealmSettingsEmail from '../ui/page-realm-settings-email'

const smtpConfigSchema = z.object({
  host: z.string().min(1, 'Host is required'),
  port: z.number().min(1).max(65535),
  username: z.string().min(1, 'Username is required'),
  password: z.string().min(1, 'Password is required'),
  from_email: z.email('Must be a valid email'),
  from_name: z.string().min(1, 'From name is required'),
  encryption: z.enum(['tls', 'starttls', 'none']),
})

export type SmtpConfigSchema = z.infer<typeof smtpConfigSchema>

export default function PageRealmSettingsEmailFeature() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const realm = realm_name ?? 'master'
  const { data, isError } = useGetSmtpConfig({ realm: realm_name })
  const { mutate: upsert } = useUpsertSmtpConfig()
  const { mutate: remove } = useDeleteSmtpConfig()
  const { data: templatesData, isLoading: templatesLoading } = useGetEmailTemplates({ realm })
  const { mutate: deleteTemplate } = useDeleteEmailTemplate()
  const { data: realmData } = useGetRealm({ realm })
  const { mutate: updateSettings } = useUpdateRealmSettings()

  const hasConfig = !!data && !isError

  const form = useForm<SmtpConfigSchema>({
    resolver: zodResolver(smtpConfigSchema),
    defaultValues: {
      host: '',
      port: 587,
      username: '',
      password: '',
      from_email: '',
      from_name: '',
      encryption: 'tls',
    },
  })

  const handleSubmit = (values: SmtpConfigSchema) => {
    if (!realm_name) return

    upsert({
      path: { realm_name },
      body: values,
    })
  }

  const handleDelete = () => {
    if (!realm_name) return

    remove(
      { path: { realm_name } },
      {
        onSuccess: () => {
          form.reset({
            host: '',
            port: 587,
            username: '',
            password: '',
            from_email: '',
            from_name: '',
            encryption: 'tls',
          })
        },
      },
    )
  }

  const handleEditTemplate = (templateId: string) => {
    navigate(EMAIL_TEMPLATE_BUILDER_URL(realm_name, templateId))
  }

  const handleCreateTemplate = () => {
    navigate(EMAIL_TEMPLATE_BUILDER_URL(realm_name, 'new'))
  }

  const handleDeleteTemplate = (templateId: string) => {
    deleteTemplate({ path: { realm_name: realm, template_id: templateId } })
  }

  const handleAssignTemplate = (field: string, templateId: string | null) => {
    if (!realm_name) return

    updateSettings({
      path: { name: realm_name },
      body: {
        [field]: templateId,
      },
    })
  }

  const realmSettings = realmData?.settings

  return (
    <PageRealmSettingsEmail
      form={form}
      config={hasConfig ? data : undefined}
      handleSubmit={handleSubmit}
      handleDelete={handleDelete}
      templates={templatesData?.data ?? []}
      templatesLoading={templatesLoading}
      onEditTemplate={handleEditTemplate}
      onCreateTemplate={handleCreateTemplate}
      onDeleteTemplate={handleDeleteTemplate}
      realmSettings={realmSettings}
      onAssignTemplate={handleAssignTemplate}
    />
  )
}
