import { useDeleteSmtpConfig, useGetSmtpConfig, useUpsertSmtpConfig } from '@/api/smtp.api'
import { RouterParams } from '@/routes/router'
import { zodResolver } from '@hookform/resolvers/zod'
import { useForm } from 'react-hook-form'
import { useParams } from 'react-router'
import {
  smtpConfigSchema,
  type SmtpConfigSchema,
} from '../schemas/smtp-config.schema'
import PageRealmSettingsSmtp from '../ui/page-realm-settings-smtp'

export default function PageRealmSettingsSmtpFeature() {
  const { realm_name } = useParams<RouterParams>()
  const { data, isError } = useGetSmtpConfig({ realm: realm_name })
  const { mutate: upsert } = useUpsertSmtpConfig()
  const { mutate: remove } = useDeleteSmtpConfig()

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

  return (
    <PageRealmSettingsSmtp
      form={form}
      config={hasConfig ? data : undefined}
      handleSubmit={handleSubmit}
      handleDelete={handleDelete}
    />
  )
}
